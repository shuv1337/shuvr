use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::widgets::{panel_contrast_fg, render_panel_shell};
use crate::app::{AppState, Mode};

fn prefix_rhs_label(bindings: &crate::config::ActionKeybinds) -> String {
    bindings
        .prefix_rhs_label()
        .unwrap_or_else(|| "unset".to_string())
}

fn keybind_label(bindings: &crate::config::ActionKeybinds) -> String {
    bindings.label().unwrap_or_else(|| "unset".to_string())
}

fn render_bottom_bar(frame: &mut Frame, area: Rect, line: Line<'_>, bg: ratatui::style::Color) {
    frame.render_widget(Clear, area);
    let buf = frame.buffer_mut();
    for x in area.x..area.x + area.width {
        buf[(x, area.y)].set_style(Style::default().bg(bg));
    }
    frame.render_widget(Paragraph::new(line), area);
}

/// Draw the persistent bottom mode bar into `bar_area` (a single reserved row):
/// a colored mode badge, an optional plain note, then key/description hint pairs
/// fit to the available width. Hints that do not fit are dropped and an ellipsis
/// is shown, so the bar never overpaints content or truncates mid-word.
fn render_mode_bar(
    frame: &mut Frame,
    bar_area: Rect,
    badge: &str,
    badge_bg: Color,
    note: Option<&str>,
    hints: &[(String, &str)],
    reserved_right: u16,
    p: &crate::app::state::Palette,
) {
    if bar_area.width == 0 || bar_area.height == 0 {
        return;
    }

    let key_style = Style::default().fg(p.accent).add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(p.overlay0);
    let badge_style = Style::default()
        .fg(panel_contrast_fg(p))
        .bg(badge_bg)
        .add_modifier(Modifier::BOLD);

    // Budget content away from any reserved right-hand strip (e.g. the
    // update-ready badge), but still fill the full-width background below.
    let total = (bar_area.width as usize).saturating_sub(reserved_right as usize);
    let mut spans = vec![Span::styled(format!(" {badge} "), badge_style)];
    let mut used = badge.chars().count() + 2;

    let gap = "   ";
    if let Some(note) = note {
        let seg = gap.len() + note.chars().count();
        if used + seg <= total {
            spans.push(Span::styled(format!("{gap}{note}"), dim));
            used += seg;
        }
    }

    for (i, (key, desc)) in hints.iter().enumerate() {
        let last = i + 1 == hints.len();
        let seg = gap.len() + key.chars().count() + 1 + desc.chars().count();
        // Reserve room for the ellipsis when more hints could still follow.
        let reserve = if last { 0 } else { 2 };
        if used + seg + reserve > total {
            if used + 2 <= total {
                spans.push(Span::styled(" …", dim));
            }
            break;
        }
        spans.push(Span::styled(gap, dim));
        spans.push(Span::styled(key.clone(), key_style));
        spans.push(Span::styled(format!(" {desc}"), dim));
        used += seg;
    }

    render_bottom_bar(frame, bar_area, Line::from(spans), p.panel_bg);
}

/// Hints for NAVIGATE mode, gated on what can actually fire right now: with no
/// workspace, only "create workspace" + help; otherwise the curated movement set
/// (the full list lives behind `?`).
fn navigate_hints(app: &AppState) -> Vec<(String, &'static str)> {
    let kb = &app.keybinds;
    if app.active.is_none() {
        return vec![
            ("⏎".to_string(), "create workspace"),
            (prefix_rhs_label(&kb.command_palette), "commands"),
            (prefix_rhs_label(&kb.help), "all keys"),
        ];
    }
    let workspace_nav = format!(
        "{} / {}",
        keybind_label(&kb.navigate.workspace_up),
        keybind_label(&kb.navigate.workspace_down)
    );
    vec![
        (workspace_nav, "workspaces"),
        ("⇥".to_string(), "panes"),
        (prefix_rhs_label(&kb.goto), "navigator"),
        (prefix_rhs_label(&kb.new_tab), "new tab"),
        (prefix_rhs_label(&kb.split_vertical), "split"),
        (prefix_rhs_label(&kb.command_palette), "commands"),
        (prefix_rhs_label(&kb.help), "all keys"),
    ]
}

/// Hints shown after the prefix key is pressed — the most useful prefix actions,
/// now including "new workspace" (previously absent).
fn prefix_hints(app: &AppState) -> Vec<(String, &'static str)> {
    let kb = &app.keybinds;
    vec![
        ("esc".to_string(), "cancel"),
        (prefix_rhs_label(&kb.command_palette), "commands"),
        (prefix_rhs_label(&kb.new_workspace), "new workspace"),
        (prefix_rhs_label(&kb.new_tab), "new tab"),
        (prefix_rhs_label(&kb.split_vertical), "split"),
        (prefix_rhs_label(&kb.workspace_picker), "workspaces"),
        (prefix_rhs_label(&kb.goto), "navigator"),
        (prefix_rhs_label(&kb.help), "keybinds"),
    ]
}

/// The persistent status bar: always tells the user which mode they are in (so
/// they know where their next keystroke goes) and what they can do right now.
/// Rendered every frame into the reserved bottom row, for every mode.
pub(super) fn render_status_bar(app: &AppState, frame: &mut Frame, bar_area: Rect) {
    if bar_area.width == 0 || bar_area.height == 0 {
        return;
    }
    let p = &app.palette;
    let prefix = crate::config::format_key_combo((app.prefix_code, app.prefix_mods));

    // Reserve room on the right for the update-ready badge so it never overpaints
    // hints mid-word (the badge is 13 cols + a 1-col gap).
    let show_update = app.update_available.is_some()
        && matches!(app.mode, Mode::Terminal | Mode::Navigate | Mode::Prefix);
    let reserved_right: u16 = if show_update { 14 } else { 0 };

    match app.mode {
        Mode::Terminal => {
            let palette = format!(
                "{prefix} {}",
                prefix_rhs_label(&app.keybinds.command_palette)
            );
            render_mode_bar(
                frame,
                bar_area,
                "TERMINAL",
                p.green,
                Some("keys go to your shell"),
                &[(palette, "commands")],
                reserved_right,
                p,
            );
        }
        Mode::Navigate => render_mode_bar(
            frame,
            bar_area,
            "NAVIGATE",
            p.accent,
            None,
            &navigate_hints(app),
            reserved_right,
            p,
        ),
        Mode::Prefix => render_mode_bar(
            frame,
            bar_area,
            "PREFIX",
            p.accent,
            None,
            &prefix_hints(app),
            reserved_right,
            p,
        ),
        Mode::Copy => {
            let select = if app.copy_mode.is_some_and(|copy_mode| copy_mode.selecting) {
                "selecting"
            } else {
                "select"
            };
            let hints: Vec<(String, &str)> = vec![
                ("h/j/k/l w/b/e".to_string(), "move"),
                ("v/space".to_string(), select),
                ("y/enter".to_string(), "copy"),
                ("q/esc".to_string(), "exit"),
            ];
            render_mode_bar(frame, bar_area, "COPY", p.accent, None, &hints, 0, p);
        }
        Mode::Resize => {
            let hints: Vec<(String, &str)> = vec![
                ("h/l".to_string(), "width"),
                ("j/k".to_string(), "height"),
                ("esc".to_string(), "done"),
            ];
            render_mode_bar(frame, bar_area, "RESIZE", p.mauve, None, &hints, 0, p);
        }
        // Modal modes own the screen with their own UI; the reserved row shows a
        // neutral badge so it never looks empty or broken.
        other => {
            let (badge, note): (&str, &str) = match other {
                Mode::Settings => ("SETTINGS", "esc close"),
                Mode::CommandPalette => ("COMMANDS", "type · ↑↓ · ↵ run · esc"),
                Mode::Navigator => ("NAVIGATOR", "esc close"),
                Mode::KeybindHelp => ("KEYBINDS", "esc close"),
                Mode::GlobalMenu | Mode::ContextMenu => ("MENU", "esc close"),
                Mode::RenameWorkspace | Mode::RenameTab | Mode::RenamePane => {
                    ("RENAME", "enter save · esc cancel")
                }
                Mode::NewLinkedWorktree | Mode::OpenExistingWorktree => ("WORKTREE", "esc cancel"),
                Mode::ConfirmClose | Mode::ConfirmRemoveWorktree => ("CONFIRM", "y / n"),
                Mode::Onboarding => ("WELCOME", "enter continue"),
                Mode::ReleaseNotes => ("RELEASE NOTES", "esc close"),
                Mode::ProductAnnouncement => ("ANNOUNCEMENT", "esc close"),
                _ => ("", ""),
            };
            if !badge.is_empty() {
                render_mode_bar(frame, bar_area, badge, p.overlay0, Some(note), &[], 0, p);
            }
        }
    }

    // Update-ready hint, right-aligned. render_mode_bar already filled the row
    // background, so this draws on top (no Clear seam) within the reserved strip.
    if show_update {
        let status = Line::from(vec![Span::styled(
            " update ready",
            Style::default()
                .fg(p.accent)
                .bg(p.panel_bg)
                .add_modifier(Modifier::BOLD),
        )]);
        let width = 13u16.min(bar_area.width);
        let status_area = Rect::new(
            bar_area.x + bar_area.width.saturating_sub(width),
            bar_area.y,
            width,
            bar_area.height,
        );
        frame.render_widget(
            Paragraph::new(status).alignment(Alignment::Right),
            status_area,
        );
    }
}

pub(super) fn render_global_launcher_menu(app: &AppState, frame: &mut Frame) {
    let rect = app.global_menu_rect();
    let Some(inner) = render_panel_shell(frame, rect, app.palette.accent, app.palette.panel_bg)
    else {
        return;
    };

    let items = app.global_menu_labels();
    for (idx, item) in items.iter().enumerate() {
        let y = inner.y + idx as u16;
        if y >= inner.y + inner.height {
            break;
        }
        let selected = idx == app.global_menu.highlighted;
        let rect = Rect::new(inner.x, y, inner.width, 1);

        let selected_style = Style::default()
            .fg(panel_contrast_fg(&app.palette))
            .bg(app.palette.accent)
            .add_modifier(Modifier::BOLD);
        let item_style = if selected {
            selected_style
        } else {
            Style::default().fg(app.palette.text)
        };
        let badge_style = if selected {
            selected_style
        } else {
            Style::default()
                .fg(app.palette.accent)
                .add_modifier(Modifier::BOLD)
        };

        let line = if app.global_menu_item_has_badge(item) {
            Line::from(vec![
                Span::styled(" ●", badge_style),
                Span::styled(format!(" {item} "), item_style),
            ])
        } else {
            Line::from(Span::styled(format!(" {item} "), item_style))
        };
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Left), rect);
    }
}

pub(super) fn render_context_menu(app: &AppState, frame: &mut Frame) {
    let Some(menu) = &app.context_menu else {
        return;
    };

    let p = &app.palette;
    let Some(menu_rect) = app.context_menu_rect() else {
        return;
    };
    let Some(inner) = render_panel_shell(frame, menu_rect, p.accent, p.panel_bg) else {
        return;
    };

    let items: Vec<ListItem> = menu
        .items()
        .iter()
        .map(|item| ListItem::new(Line::from(*item)))
        .collect();
    let list = List::new(items)
        .style(Style::default().fg(p.text))
        .highlight_style(
            Style::default()
                .bg(p.accent)
                .fg(panel_contrast_fg(p))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");
    let mut state = ListState::default().with_selected(Some(menu.list.highlighted));
    frame.render_stateful_widget(list, inner, &mut state);
}
