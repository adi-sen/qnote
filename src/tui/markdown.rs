use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Lightweight markdown rendering - just basic formatting
pub fn markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
    markdown
        .lines()
        .map(|line| {
            if let Some(header) = line.strip_prefix("# ") {
                Line::from(Span::styled(
                    header.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if let Some(header) = line.strip_prefix("## ") {
                Line::from(Span::styled(
                    header.to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("- ") || line.starts_with("* ") {
                Line::from(vec![
                    Span::styled("  • ".to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(
                        line.trim_start_matches("- ")
                            .trim_start_matches("* ")
                            .to_string(),
                    ),
                ])
            } else if line.starts_with("```") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Gray),
                ))
            } else {
                Line::from(line.to_string())
            }
        })
        .collect()
}
