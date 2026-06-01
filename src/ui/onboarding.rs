use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::widgets::{
    action_button_width, modal_stack_areas, panel_contrast_fg, render_action_button,
    render_modal_shell,
};
use crate::app::AppState;

pub(super) fn render_onboarding_overlay(app: &AppState, frame: &mut Frame, area: Rect) {
    super::dim_background(frame, area);
    render_onboarding_welcome(app, frame, area);
}

pub(crate) fn onboarding_welcome_continue_rect(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y,
        action_button_width(Some("↵"), "continue"),
        1,
    )
}

fn render_onboarding_welcome(app: &AppState, frame: &mut Frame, area: Rect) {
    let Some(inner) = render_modal_shell(frame, area, 68, 17, &app.palette) else {
        return;
    };
    if inner.height < 12 {
        return;
    }

    let stack = modal_stack_areas(inner, 2, 0, 1, 1);
    let header_rows =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas::<2>(stack.header);
    let content_rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas::<4>(stack.content);

    frame.render_widget(
        Paragraph::new("  shuvr").style(
            Style::default()
                .fg(app.palette.mauve)
                .add_modifier(Modifier::BOLD),
        ),
        header_rows[0],
    );
    frame.render_widget(
        Paragraph::new("  terminal workspace manager for coding agents")
            .style(Style::default().fg(app.palette.overlay0)),
        header_rows[1],
    );

    frame.render_widget(
        Paragraph::new("  shuvr watches your agents so you don't have to").style(
            Style::default()
                .fg(app.palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        content_rows[0],
    );

    frame.render_widget(
        Paragraph::new(
            "  Get notified when Claude, Codex, or Pi finishes a task.\n  Resume your session across restarts. Every tab and pane\n  is a real terminal. Run a coding agent and watch its\n  state in the sidebar.",
        )
        .style(Style::default().fg(app.palette.overlay1)),
        content_rows[1],
    );

    let prefix = crate::config::format_key_combo((app.prefix_code, app.prefix_mods));
    let accent = Style::default()
        .fg(app.palette.accent)
        .add_modifier(Modifier::BOLD);
    let key_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(prefix, accent),
        Span::styled(
            " prefix, then a letter. ",
            Style::default().fg(app.palette.overlay1),
        ),
        Span::styled("?", accent),
        Span::styled(" shows all keys", Style::default().fg(app.palette.overlay1)),
    ]);
    frame.render_widget(Paragraph::new(key_line), content_rows[2]);

    frame.render_widget(
        Paragraph::new(
            "  Mouse works too: click, drag panes, right-click for actions.\n  Press -> to set up agent integrations (optional).",
        )
        .style(Style::default().fg(app.palette.overlay1)),
        content_rows[3],
    );

    let continue_rect = onboarding_welcome_continue_rect(stack.actions.unwrap_or_default());
    render_action_button(
        frame,
        continue_rect,
        Some("↵"),
        "continue",
        Style::default()
            .fg(panel_contrast_fg(&app.palette))
            .bg(app.palette.accent)
            .add_modifier(Modifier::BOLD),
    );
}
