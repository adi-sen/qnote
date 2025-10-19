use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

const H1_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const H2_STYLE: Style = Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD);
const BULLET_STYLE: Style = Style::new().fg(Color::Yellow);
const CODE_STYLE: Style = Style::new().fg(Color::Gray);

/// Lightweight markdown rendering - just basic formatting
pub fn markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
    if markdown.is_empty() {
        return Vec::new();
    }

    markdown
        .lines()
        .map(|line| {
            line.strip_prefix("# ")
                .map(|header| Line::from(Span::styled(header.to_string(), H1_STYLE)))
                .or_else(|| {
                    line.strip_prefix("## ")
                        .map(|header| Line::from(Span::styled(header.to_string(), H2_STYLE)))
                })
                .unwrap_or_else(|| {
                    if line.starts_with("- ") || line.starts_with("* ") {
                        Line::from(vec![
                            Span::styled("  • ".to_string(), BULLET_STYLE),
                            Span::raw(
                                line.trim_start_matches("- ")
                                    .trim_start_matches("* ")
                                    .to_string(),
                            ),
                        ])
                    } else if line.starts_with("```") {
                        Line::from(Span::styled(line.to_string(), CODE_STYLE))
                    } else {
                        Line::from(line.to_string())
                    }
                })
        })
        .collect()
}
