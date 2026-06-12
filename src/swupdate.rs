use embedded_graphics::Drawable;
use embedded_graphics::{
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::BinaryColor,
	prelude::Point,
	text::{Alignment, Text},
};

use crate::{FramebufferTarget, SubApp};

#[derive(Copy, Clone)]
pub struct SWUpdate;

impl SubApp for SWUpdate {
	fn handle_events(&mut self, event: crate::JoystickEvents) {
		println!("Handling events from SWUpdate {event:?}");
	}
	fn display(&self, target: &mut FramebufferTarget) {
		let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		Text::with_alignment(
			"SWUpdate",
			Point::new(200, 20),
			text_style,
			Alignment::Center,
		)
		.draw(target)
		.unwrap();
	}
	fn update(&mut self) {}
}
