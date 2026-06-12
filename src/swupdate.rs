use embedded_graphics::Drawable;
use embedded_graphics::{
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::BinaryColor,
	prelude::Point,
	text::{Alignment, Text},
};

use crate::SubApp;

#[derive(Copy, Clone)]
pub struct SWUpdate;

impl SubApp for SWUpdate {
	fn handle_events(&mut self, event: crate::JoystickEvents) {
		println!("Handling events from SWUpdate {event:?}");
	}
	fn display<D>(&self, target: &mut D)
	where
		D: embedded_graphics::prelude::DrawTarget<
				Color = embedded_graphics::pixelcolor::BinaryColor,
			>,
		<D as embedded_graphics::prelude::DrawTarget>::Error: std::fmt::Debug,
	{
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
}
