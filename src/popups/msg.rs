use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState,
};
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	strings, ui,
};
use crossterm::event::Event;
use ratatui::{
	layout::{Alignment, Rect},
	text::Span,
	widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
	Frame,
};
use ui::style::SharedTheme;

pub struct MsgPopup {
	title: String,
	msg: String,
	visible: bool,
	theme: SharedTheme,
	key_config: SharedKeyConfig,
}

use anyhow::Result;

impl DrawableComponent for MsgPopup {
	fn draw(&self, f: &mut Frame, _rect: Rect) -> Result<()> {
		if !self.visible {
			return Ok(());
		}

		// determine the maximum width of text block
		let lens = self
			.msg
			.split('\n')
			.map(str::len)
			.collect::<Vec<usize>>();
		let mut max = lens.iter().max().expect("max") + 2;
		if max > std::u16::MAX as usize {
			max = std::u16::MAX as usize;
		}
		let mut width = u16::try_from(max)
			.expect("can't fail due to check above");
		// dont overflow screen, and dont get too narrow
		if width > f.size().width {
			width = f.size().width;
		} else if width < 60 {
			width = 60;
		}

		let area = ui::centered_rect_absolute(width, 25, f.size());
		f.render_widget(Clear, area);
		f.render_widget(
			Paragraph::new(self.msg.clone())
				.block(
					Block::default()
						.title(Span::styled(
							self.title.as_str(),
							self.theme.text_danger(),
						))
						.borders(Borders::ALL)
						.border_type(BorderType::Thick),
				)
				.alignment(Alignment::Left)
				.wrap(Wrap { trim: true }),
			area,
		);

		Ok(())
	}
}

impl Component for MsgPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		_force_all: bool,
	) -> CommandBlocking {
		out.push(CommandInfo::new(
			strings::commands::close_msg(&self.key_config),
			true,
			self.visible,
		));

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.visible {
			if let Event::Key(e) = ev {
				if key_match(e, self.key_config.keys.enter) {
					self.hide();
				}
			}
			Ok(EventState::Consumed)
		} else {
			Ok(EventState::NotConsumed)
		}
	}

	fn is_visible(&self) -> bool {
		self.visible
	}

	fn hide(&mut self) {
		self.visible = false;
	}

	fn show(&mut self) -> Result<()> {
		self.visible = true;

		Ok(())
	}
}

impl MsgPopup {
	pub fn new(env: &Environment) -> Self {
		Self {
			title: String::new(),
			msg: String::new(),
			visible: false,
			theme: env.theme.clone(),
			key_config: env.key_config.clone(),
		}
	}

	///
	pub fn show_error(&mut self, msg: &str) -> Result<()> {
		self.title = strings::msg_title_error(&self.key_config);
		self.msg = msg.to_string();
		self.show()?;

		Ok(())
	}

	///
	pub fn show_info(&mut self, msg: &str) -> Result<()> {
		self.title = strings::msg_title_info(&self.key_config);
		self.msg = msg.to_string();
		self.show()?;

		Ok(())
	}
}
