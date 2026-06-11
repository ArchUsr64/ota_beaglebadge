use embedded_graphics::{
	draw_target::DrawTarget,
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::BinaryColor,
	prelude::*,
	text::Text,
};

use evdev::Device;

struct FramebufferTarget<'a> {
	data: &'a mut [u8],
	width: u32,
	height: u32,
	bpp: u32,
}

impl FramebufferTarget<'_> {
	const WHITE_BYTE: u8 = 0xFF;
	const BLACK_BYTE: u8 = 0x00;
}

impl<'a> DrawTarget for FramebufferTarget<'a> {
	type Color = BinaryColor;
	type Error = std::convert::Infallible;

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = Pixel<Self::Color>>,
	{
		for Pixel(point, color) in pixels.into_iter() {
			if point.x >= 0
				&& point.x < self.width as i32
				&& point.y >= 0
				&& point.y < self.height as i32
			{
				let index = (point.y as usize * self.width as usize) + point.x as usize;

				if index < self.data.len() {
					for i in 0..self.bpp {
						self.data[index * 4 + i as usize] = match color {
							BinaryColor::On => FramebufferTarget::BLACK_BYTE,
							BinaryColor::Off => FramebufferTarget::WHITE_BYTE,
						};
					}
				}
			}
		}
		Ok(())
	}
}

impl<'a> OriginDimensions for FramebufferTarget<'a> {
	fn size(&self) -> Size {
		Size::new(self.width, self.height)
	}
}

fn main() {
	let fb = linuxfb::Framebuffer::new("/dev/fb0").unwrap();

	let mut data = fb.map().unwrap();

	let mut target = FramebufferTarget {
		data: &mut data,
		width: fb.get_size().0,
		height: fb.get_size().1,
		bpp: fb.get_bytes_per_pixel(),
	};

	target.clear(BinaryColor::Off);

	let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
	Text::new("Welcome to BeagleBadge!", Point::new(100, 20), text_style)
		.draw(&mut target)
		.unwrap();

	let mut device = Device::open("/dev/input/event0").unwrap();

	println!(
		"Listening for joystick events on: {}",
		device.name().unwrap_or("Unknown Device")
	);

	loop {
		let events = device.fetch_events().expect("Failed to read events");

		for event in events {
			match event.destructure() {
				e => {
					dbg!(e);
				}
			}
		}
	}
}
