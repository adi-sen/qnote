use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};

use crate::config::ThemeConfig;

/// Renders markdown to styled lines using theme colors
pub fn markdown_to_lines(markdown: &str, theme: &ThemeConfig) -> Vec<Line<'static>> {
	if markdown.is_empty() {
		return Vec::new();
	}

	let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
	let parser = Parser::new_ext(markdown, opts);
	Renderer::new(theme).render(parser)
}

struct Renderer {
	lines:               Vec<Line<'static>>,
	current_line:        Vec<Span<'static>>,
	styles:              Vec<Style>,
	in_code_block:       bool,
	in_list:             bool,
	list_level:          usize,
	in_blockquote:       bool,
	item_needs_prefix:   bool,
	h1_color:            Color,
	h2_color:            Color,
	h3_color:            Color,
	h4_h6_color:         Color,
	code_color:          Color,
	code_block_color:    Color,
	link_color:          Color,
	emphasis_color:      Color,
	strong_color:        Color,
	strikethrough_color: Color,
	blockquote_color:    Color,
}

impl Renderer {
	fn new(theme: &ThemeConfig) -> Self {
		Self {
			lines:               Vec::new(),
			current_line:        Vec::new(),
			styles:              Vec::new(),
			in_code_block:       false,
			in_list:             false,
			list_level:          0,
			in_blockquote:       false,
			item_needs_prefix:   false,
			h1_color:            *theme.h1,
			h2_color:            *theme.h2,
			h3_color:            *theme.h3,
			h4_h6_color:         *theme.h4_h6,
			code_color:          *theme.code,
			code_block_color:    *theme.code_block,
			link_color:          *theme.link,
			emphasis_color:      *theme.emphasis,
			strong_color:        *theme.strong,
			strikethrough_color: *theme.strikethrough,
			blockquote_color:    *theme.blockquote,
		}
	}
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
					HeadingLevel::H1 => self.h1_color,
					HeadingLevel::H2 => self.h2_color,
					HeadingLevel::H3 => self.h3_color,
					_ => self.h4_h6_color,
				};
				self.push_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
			}
			Tag::BlockQuote(_) => {
				self.finish_line();
				self.in_blockquote = true;
				self.push_style(Style::default().fg(self.blockquote_color).add_modifier(Modifier::ITALIC));
			}
			Tag::CodeBlock(_) => {
				self.finish_line();
				self.in_code_block = true;
				self.push_style(Style::default().fg(self.code_block_color));
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
			Tag::Strong => self.push_style(Style::default().fg(self.strong_color).add_modifier(Modifier::BOLD)),
			Tag::Emphasis => self.push_style(Style::default().fg(self.emphasis_color).add_modifier(Modifier::ITALIC)),
			Tag::Strikethrough => {
				self.push_style(Style::default().fg(self.strikethrough_color).add_modifier(Modifier::CROSSED_OUT))
			}
			Tag::Link { .. } => {
				self.push_style(Style::default().fg(self.link_color).add_modifier(Modifier::UNDERLINED));
				self.push_span("[");
			}
			Tag::Image { .. } => self.push_span("[Image: "),
			_ => {}
		}
	}

	fn end_tag(&mut self, tag: TagEnd) {
		match tag {
			TagEnd::Paragraph if !self.in_list || self.in_blockquote => self.finish_line(),
			TagEnd::Heading(_) => {
				self.finish_line();
				self.pop_style();
			}
			TagEnd::BlockQuote(_) | TagEnd::CodeBlock => {
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
		self.current_line.push(Span::styled(format!("`{code}`"), Style::default().fg(self.code_color)));
	}
}
