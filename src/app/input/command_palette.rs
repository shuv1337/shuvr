use crossterm::event::{KeyCode, KeyModifiers};

use super::navigate::{execute_navigate_action_in_context, ActionContext, NavigateAction};
use crate::app::{App, AppState, Mode};
use crate::input::TerminalKey;

/// One selectable command in the palette.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PaletteEntry {
    pub label: &'static str,
    pub action: NavigateAction,
    /// Hidden when there is no active workspace (the action would no-op).
    pub needs_workspace: bool,
}

const fn entry(label: &'static str, action: NavigateAction, needs_workspace: bool) -> PaletteEntry {
    PaletteEntry {
        label,
        action,
        needs_workspace,
    }
}

/// The full command registry, in display order. Filtered by query and by whether
/// a workspace exists.
pub(crate) const PALETTE_ENTRIES: &[PaletteEntry] = &[
    entry("New workspace", NavigateAction::NewWorkspace, false),
    entry("New tab", NavigateAction::NewTab, true),
    entry("Split vertical", NavigateAction::SplitVertical, true),
    entry("Split horizontal", NavigateAction::SplitHorizontal, true),
    entry("Close pane", NavigateAction::ClosePane, true),
    entry("Zoom pane", NavigateAction::Zoom, true),
    entry("Rename pane", NavigateAction::RenamePane, true),
    entry("Resize panes", NavigateAction::EnterResizeMode, true),
    entry("Copy mode", NavigateAction::CopyMode, true),
    entry("Rename tab", NavigateAction::RenameTab, true),
    entry("Close tab", NavigateAction::CloseTab, true),
    entry("Rename workspace", NavigateAction::RenameWorkspace, true),
    entry("Close workspace", NavigateAction::CloseWorkspace, true),
    entry("New worktree", NavigateAction::NewWorktree, true),
    entry("Open worktree", NavigateAction::OpenWorktree, true),
    entry("Session navigator", NavigateAction::OpenNavigator, false),
    entry("Settings", NavigateAction::Settings, false),
    entry("Keybindings", NavigateAction::Help, false),
    entry("Reload config", NavigateAction::ReloadConfig, false),
    entry("Detach", NavigateAction::Detach, false),
];

impl AppState {
    /// The palette entries visible right now, filtered by the current query and
    /// by whether a workspace exists.
    pub(crate) fn command_palette_visible_entries(&self) -> Vec<PaletteEntry> {
        let query = self.command_palette.query.to_lowercase();
        let has_workspace = self.active.is_some();
        PALETTE_ENTRIES
            .iter()
            .copied()
            .filter(|entry| has_workspace || !entry.needs_workspace)
            .filter(|entry| query.is_empty() || entry.label.to_lowercase().contains(&query))
            .collect()
    }

    fn command_palette_move(&mut self, delta: isize) {
        let len = self.command_palette_visible_entries().len();
        if len == 0 {
            self.command_palette.selected = 0;
            return;
        }
        let current = self.command_palette.selected.min(len - 1) as isize;
        self.command_palette.selected = (current + delta).rem_euclid(len as isize) as usize;
    }

    fn command_palette_reset_selection(&mut self) {
        self.command_palette.selected = 0;
        self.command_palette.scroll = 0;
    }
}

impl App {
    pub(crate) fn handle_command_palette_key(&mut self, raw_key: TerminalKey) {
        let key = raw_key.as_key_event();
        let control = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => self.leave_command_palette(),
            KeyCode::Enter => {
                let entries = self.state.command_palette_visible_entries();
                match entries.get(self.state.command_palette.selected).copied() {
                    Some(entry) => self.run_command_palette_entry(entry),
                    None => self.leave_command_palette(),
                }
            }
            KeyCode::Up => self.state.command_palette_move(-1),
            KeyCode::Down => self.state.command_palette_move(1),
            KeyCode::Char('p') if control => self.state.command_palette_move(-1),
            KeyCode::Char('n') if control => self.state.command_palette_move(1),
            KeyCode::Char('u') if control => {
                self.state.command_palette.query.clear();
                self.state.command_palette_reset_selection();
            }
            KeyCode::Backspace => {
                self.state.command_palette.query.pop();
                self.state.command_palette_reset_selection();
            }
            KeyCode::Char(c) if key.modifiers.difference(KeyModifiers::SHIFT).is_empty() => {
                self.state.command_palette.query.push(c);
                self.state.command_palette_reset_selection();
            }
            _ => {}
        }
    }

    fn leave_command_palette(&mut self) {
        self.state.mode = if self.state.active.is_some() {
            Mode::Terminal
        } else {
            Mode::Navigate
        };
    }

    fn run_command_palette_entry(&mut self, entry: PaletteEntry) {
        // Close the palette back to the resting mode the action expects, then
        // run it through the same execution path as Navigate mode so all of its
        // mode transitions (rename dialogs, resize mode, request flags, …) behave
        // exactly as if triggered by a key.
        self.leave_command_palette();
        execute_navigate_action_in_context(
            &mut self.state,
            &mut self.terminal_runtimes,
            entry.action,
            ActionContext::Navigate,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;

    #[test]
    fn empty_state_hides_pane_actions() {
        let state = AppState::test_new();
        let labels: Vec<&str> = state
            .command_palette_visible_entries()
            .iter()
            .map(|entry| entry.label)
            .collect();
        assert!(labels.contains(&"New workspace"));
        assert!(!labels.contains(&"Split vertical"));
        assert!(labels.contains(&"Settings"));
    }

    #[test]
    fn open_command_palette_action_switches_mode() {
        use super::super::navigate::execute_navigate_action;
        let mut state = AppState::test_new();
        execute_navigate_action(&mut state, NavigateAction::OpenCommandPalette);
        assert_eq!(state.mode, Mode::CommandPalette);
    }

    #[test]
    fn query_filters_entries_case_insensitively() {
        let mut state = AppState::test_new();
        state.command_palette.query = "WORK".to_string();
        let labels: Vec<&str> = state
            .command_palette_visible_entries()
            .iter()
            .map(|entry| entry.label)
            .collect();
        assert!(labels.contains(&"New workspace"));
        assert!(!labels.contains(&"Settings"));
    }
}
