use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};

const HEADING1: Color = Color::Cyan;
const HEADING2: Color = Color::Blue;
const HEADING3: Color = Color::LightBlue;
const CODE: Color = Color::Yellow;
const QUOTE: Color = Color::Gray;
const LINK: Color = Color::Blue;
const STRIKE: Color = Color::DarkGray;

/// Renders markdown to styled lines.
pub fn markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
	if markdown.is_empty() {
		return Vec::new();
	}

	let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
	let parser = Parser::new_ext(markdown, opts);
	Renderer::default().render(parser)
}

#[derive(Default)]
struct Renderer {
	lines:             Vec<Line<'static>>,
	current_line:      Vec<Span<'static>>,
	styles:            Vec<Style>,
	in_code_block:     bool,
	in_list:           bool,
	list_level:        usize,
	in_blockquote:     bool,
	item_needs_prefix: bool,
}

impl Renderer {
	fn style(&self) -> Style { self.styles.last().copied().unwrap_or_default() }

	fn push_style(&mut self, style: Style) { self.styles.push(self.style().patch(style)); }

	fn pop_style(&mut self) {
		if self.styles.len() > 1 {
			self.styles.pop();
		}
	}

	fn push_span(&mut self, text: impl Into<String>) {
		let text = text.into();
		if !text.is_empty() {
			self.current_line.push(Span::styled(text, self.style()));
		}
	}

	fn finish_line(&mut self) {
		if !self.current_line.is_empty() {
			self.lines.push(Line::from(std::mem::take(&mut self.current_line)));
		}
	}

	fn render(mut self, parser: Parser) -> Vec<Line<'static>> {
		for event in parser {
			match event {
				Event::Start(tag) => self.start_tag(tag),
				Event::End(tag) => self.end_tag(tag),
				Event::Text(text) => self.text(text.into()),
				Event::Code(code) => self.inline_code(code.into()),
				Event::SoftBreak => self.push_span(" "),
				Event::HardBreak => self.finish_line(),
				Event::Rule => {
					self.finish_line();
					self.lines.push(Line::from(Span::styled("─".repeat(80), Style::default().fg(Color::DarkGray))));
				}
				Event::TaskListMarker(checked) => {
					if self.item_needs_prefix {
						let indent = "  ".repeat(self.list_level.saturating_sub(1));
						let marker = if checked { "[✓] " } else { "[ ] " };
						self.current_line.push(Span::raw(format!("{indent}{marker}")));
						self.item_needs_prefix = false;
					}
				}
				_ => {}
			}
		}
		self.finish_line();
		self.lines
	}

	fn start_tag(&mut self, tag: Tag) {
		match tag {
			Tag::Paragraph => {}
			Tag::Heading { level, .. } => {
				self.finish_line();
				let color = match level {
					HeadingLevel::H1 => HEADING1,
					HeadingLevel::H2 => HEADING2,
					HeadingLevel::H3 => HEADING3,
					_ => Color::Reset,
				};
				self.push_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
			}
			Tag::BlockQuote(_) => {
				self.finish_line();
				self.in_blockquote = true;
				self.push_style(Style::default().fg(QUOTE).add_modifier(Modifier::ITALIC));
			}
			Tag::CodeBlock(_) => {
				self.finish_line();
				self.in_code_block = true;
				self.push_style(Style::default().fg(CODE));
			}
			Tag::List(_) => {
				if !self.in_list {
					self.finish_line();
				}
				self.in_list = true;
				self.list_level += 1;
			}
			Tag::Item => {
				self.finish_line();
				self.item_needs_prefix = true;
			}
			Tag::Strong => self.push_style(Style::default().add_modifier(Modifier::BOLD)),
			Tag::Emphasis => self.push_style(Style::default().add_modifier(Modifier::ITALIC)),
			Tag::Strikethrough => self.push_style(Style::default().fg(STRIKE).add_modifier(Modifier::CROSSED_OUT)),
			Tag::Link { .. } => {
				self.push_style(Style::default().fg(LINK).add_modifier(Modifier::UNDERLINED));
				self.push_span("[");
			}
			Tag::Image { .. } => self.push_span("[Image: "),
			_ => {}
		}
	}

	fn end_tag(&mut self, tag: TagEnd) {
		match tag {
			TagEnd::Paragraph if !self.in_list || self.in_blockquote => self.finish_line(),
			TagEnd::Heading(_) | TagEnd::BlockQuote(_) | TagEnd::CodeBlock => {
				self.pop_style();
				self.finish_line();
				if matches!(tag, TagEnd::BlockQuote(_)) {
					self.in_blockquote = false;
				} else if matches!(tag, TagEnd::CodeBlock) {
					self.in_code_block = false;
				}
			}
			TagEnd::List(_) => {
				self.list_level = self.list_level.saturating_sub(1);
				if self.list_level == 0 {
					self.in_list = false;
					self.finish_line();
				}
			}
			TagEnd::Item => self.finish_line(),
			TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough => self.pop_style(),
			TagEnd::Link => {
				self.push_span("]");
				self.pop_style();
			}
			TagEnd::Image => self.push_span("]"),
			_ => {}
		}
	}

	fn text(&mut self, text: String) {
		if self.item_needs_prefix && self.in_list {
			let indent = "  ".repeat(self.list_level.saturating_sub(1));
			self.current_line.push(Span::raw(format!("{indent}• ")));
			self.item_needs_prefix = false;
		}

		if self.in_code_block {
			for line in text.split('\n') {
				self.lines.push(Line::from(Span::styled(format!("  {line}"), self.style())));
			}
		} else if self.in_blockquote {
			self.push_span(format!("│ {text}"));
		} else {
			self.push_span(text);
		}
	}

	fn inline_code(&mut self, code: String) {
		self.current_line.push(Span::styled(format!("`{code}`"), Style::default().fg(CODE)));
	}
}
