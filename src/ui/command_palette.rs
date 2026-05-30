use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::widgets::{panel_contrast_fg, render_modal_shell};
use crate::app::AppState;

const PALETTE_WIDTH: u16 = 58;

fn palette_height(app: &AppState) -> u16 {
    let entries = app.command_palette_visible_entries().len() as u16;
    // header (2) + search (1) + two blank separators + list rows.
    entries.saturating_add(6).clamp(9, 22)
}

pub(super) fn render_command_palette_overlay(app: &AppState, frame: &mut Frame) {
    super::dim_background(frame, frame.area());
    let p = &app.palette;
    let height = palette_height(app);
    let Some(inner) = render_modal_shell(frame, frame.area(), PALETTE_WIDTH, height, p) else {
        return;
    };
    if inner.height < 4 || inner.width < 12 {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "commands",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        ))),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            "type to filter · ↑↓ move · ↵ run · esc close",
            Style::default().fg(p.overlay0),
        )),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );

    let search = Line::from(vec![
        Span::styled(
            "› ",
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app.command_palette.query.clone(),
            Style::default().fg(p.text),
        ),
        Span::styled("▏", Style::default().fg(p.accent)),
    ]);
    frame.render_widget(
        Paragraph::new(search),
        Rect::new(inner.x, inner.y + 3, inner.width, 1),
    );

    let list_y = inner.y + 5;
    let list_bottom = inner.y + inner.height;
    if list_y >= list_bottom {
        return;
    }
    let rows = (list_bottom - list_y) as usize;

    let entries = app.command_palette_visible_entries();
    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  no matching commands",
                Style::default().fg(p.overlay0),
            )),
            Rect::new(inner.x, list_y, inner.width, 1),
        );
        return;
    }

    let selected = app.command_palette.selected.min(entries.len() - 1);
    let start = if selected >= rows {
        selected + 1 - rows
    } else {
        0
    };
    let width = inner.width as usize;

    for (offset, entry) in entries.iter().enumerate().skip(start).take(rows) {
        let y = list_y + (offset - start) as u16;
        let is_selected = offset == selected;
        let marker = if is_selected { "› " } else { "  " };
        let text = format!("{marker}{}", entry.label);
        let padded = format!("{text:<width$}");
        let style = if is_selected {
            Style::default()
                .fg(panel_contrast_fg(p))
                .bg(p.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.text)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(padded, style))),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }
}
