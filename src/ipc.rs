use std::fs;
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SocketFileIdentity {
    dev: u64,
    ino: u64,
}

pub(crate) fn prepare_socket_path(
    path: &Path,
    busy_message: impl FnOnce(&Path) -> String,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    if !file_type.is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to remove non-socket {} at {} while preparing socket",
                describe_file_type(&file_type),
                path.display(),
            ),
        ));
    }

    match UnixStream::connect(path) {
        Ok(_) => {
            return Err(io::Error::new(io::ErrorKind::AddrInUse, busy_message(path)));
        }
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::NotFound
                    | io::ErrorKind::TimedOut
            ) => {}
        Err(err) => return Err(err),
    }

    if let Err(err) = fs::remove_file(path) {
        if err.kind() != io::ErrorKind::NotFound {
            return Err(err);
        }
    }

    Ok(())
}

fn describe_file_type(file_type: &fs::FileType) -> &'static str {
    if file_type.is_dir() {
        "directory"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_symlink() {
        "symlink"
    } else {
        "path"
    }
}

pub(crate) fn socket_file_identity(path: &Path) -> io::Result<SocketFileIdentity> {
    let metadata = fs::metadata(path)?;
    Ok(SocketFileIdentity {
        dev: metadata.dev(),
        ino: metadata.ino(),
    })
}

pub(crate) fn remove_socket_file_if_owned(
    path: &Path,
    identity: SocketFileIdentity,
) -> io::Result<()> {
    let current = match socket_file_identity(path) {
        Ok(current) => current,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };

    if current != identity {
        return Ok(());
    }

    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

pub(crate) fn restrict_socket_permissions(path: &Path, mode: u32) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    fn unique_test_dir(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("shuvr-ipc-{name}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn prepare_socket_path_refuses_regular_file() {
        let dir = unique_test_dir("regular-file");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shuvr.sock");
        fs::write(&path, b"not a socket").unwrap();

        let result = prepare_socket_path(&path, |_| "busy".to_string());

        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(message.contains("non-socket file"));
        assert!(message.contains(path.to_string_lossy().as_ref()));
        assert!(path.exists(), "regular file should not be removed");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prepare_socket_path_refuses_directory() {
        let dir = unique_test_dir("directory");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shuvr.sock");
        fs::create_dir_all(&path).unwrap();

        let result = prepare_socket_path(&path, |_| "busy".to_string());

        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(message.contains("non-socket directory"));
        assert!(path.is_dir(), "directory should not be removed");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prepare_socket_path_rejects_live_socket() {
        let dir = unique_test_dir("live-socket");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shuvr.sock");
        let _listener = UnixListener::bind(&path).unwrap();

        let result = prepare_socket_path(&path, |path| format!("busy: {}", path.display()));

        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AddrInUse);
        assert!(err.to_string().contains("busy:"));
        assert!(path.exists(), "live socket should not be removed");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prepare_socket_path_removes_stale_socket() {
        let dir = unique_test_dir("stale-socket");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shuvr.sock");
        {
            let _listener = UnixListener::bind(&path).unwrap();
        }

        prepare_socket_path(&path, |_| "busy".to_string()).unwrap();

        assert!(!path.exists());
        let _ = fs::remove_dir_all(dir);
    }
}
