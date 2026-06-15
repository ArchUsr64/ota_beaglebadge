use std::os::unix::net::UnixStream;
use std::io::{Read, ErrorKind};

use embedded_graphics::{
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::BinaryColor,
	prelude::*,
	primitives::{PrimitiveStyle, Rectangle},
	text::{Alignment, Text},
};

use crate::{FramebufferTarget, SubApp, JoystickEvents};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ProgressMsg {
	magic: u32,
	status: u32,
	dwl_percent: u32,
	dwl_bytes: u64,
	nsteps: u32,
	cur_step: u32,
	cur_percent: u32,
	cur_image: [u8; 256],
	hnd_name: [u8; 64],
	source: i32,
	infolen: u32,
	info: [u8; 2048],
}

pub struct SWUpdate {
	progress: f32,
	status: String,
	stream: Option<UnixStream>,
}

impl Default for SWUpdate {
	fn default() -> Self {
		Self {
			progress: 0.0,
			status: "Idle".to_string(),
			stream: None,
		}
	}
}

impl SubApp for SWUpdate {
	fn handle_events(&mut self, _event: JoystickEvents) {
		// No special handling needed
	}

	fn update(&mut self) {
		if self.stream.is_none() {
			if let Ok(stream) = UnixStream::connect("/tmp/swupdateprog") {
				let _ = stream.set_nonblocking(true);
				self.stream = Some(stream);
				self.status = "Connected".to_string();
			} else {
				self.status = "Waiting...".to_string();
				return;
			}
		}

		if let Some(ref mut stream) = self.stream {
			let mut buf = [0u8; std::mem::size_of::<ProgressMsg>()];
			match stream.read_exact(&mut buf) {
				Ok(_) => {
					let msg: ProgressMsg = unsafe { std::ptr::read(buf.as_ptr() as *const _) };
					self.progress = msg.cur_percent as f32;
					self.status = format!("Status: {}", msg.status);
				}
				Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
					// No data yet
				}
				Err(_) => {
					self.stream = None;
					self.status = "Disconnected".to_string();
				}
			}
		}
	}

	fn display(&self, target: &mut FramebufferTarget) {
		// Clear screen
		target.clear(BinaryColor::Off).unwrap();

		// Draw border
		Rectangle::new(Point::new(5, 5), Size::new(390, 290))
			.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
			.draw(target)
			.unwrap();

		// Title
		let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		Text::with_alignment(
			"SWUPDATE STATUS",
			Point::new(200, 30),
			text_style,
			Alignment::Center,
		)
		.draw(target)
		.unwrap();

		// Status
		Text::new(&format!("Status: {}", self.status), Point::new(20, 80), text_style)
			.draw(target)
			.unwrap();

		// Progress bar
		draw_progress_bar(target, 20, 100, 360, 30, self.progress);

		// Footer
		Text::with_alignment(
			"Press Back to exit",
			Point::new(200, 275),
			text_style,
			Alignment::Center,
		)
		.draw(target)
		.unwrap();
	}
}

fn draw_progress_bar(
	target: &mut FramebufferTarget,
	x: i32,
	y: i32,
	width: u32,
	height: u32,
	percentage: f32,
) {
	// Draw outline
	Rectangle::new(Point::new(x, y), Size::new(width, height))
		.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
		.draw(target)
		.unwrap();

	// Draw filled bar
	let fill_width = ((width - 4) as f32 * (percentage / 100.0))
		.round()
		.clamp(0.0, (width - 4) as f32) as u32;

	if fill_width > 0 {
		Rectangle::new(Point::new(x + 2, y + 2), Size::new(fill_width, height - 4))
			.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
			.draw(target)
			.unwrap();
	}
}
