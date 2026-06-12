mod swupdate;

#[derive(Debug)]
enum JoystickEvents {
	Up,
	Down,
	Left,
	Right,
	Select,
}

trait SubApp {
	fn handle_events(&mut self, event: JoystickEvents);
	fn display<D>(&self, target: &mut D)
	where
		D: DrawTarget<Color = BinaryColor>,
		<D as DrawTarget>::Error: std::fmt::Debug;
}

use embedded_graphics::{
	draw_target::DrawTarget,
	image::{Image, ImageRaw},
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::{BinaryColor, raw::BigEndian},
	prelude::*,
	primitives::{PrimitiveStyle, Rectangle},
	text::{Alignment, Text},
};

use evdev::{Device, EventSummary, KeyCode};

#[derive(Debug)]
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

struct App<'a, T: SubApp> {
	selection: usize,
	logos: [Logo<'a>; 6],
	subapps: [Option<T>; 6],
	inside_subapp: bool,
}

impl<'a, T: SubApp + Copy> Default for App<'a, T> {
	fn default() -> Self {
		Self {
			logos: [
				Logo::from_pbm(Point::new(40, 60), include_bytes!("../res/swupdate.pbm")),
				Logo::from_pbm(Point::new(160, 60), include_bytes!("../res/schedule.pbm")),
				Logo::from_pbm(Point::new(280, 60), include_bytes!("../res/guestbook.pbm")),
				Logo::from_pbm(Point::new(40, 180), include_bytes!("../res/snake.pbm")),
				Logo::from_pbm(Point::new(160, 180), include_bytes!("../res/specs.pbm")),
				Logo::from_pbm(Point::new(280, 180), include_bytes!("../res/sensors.pbm")),
			],
			selection: 0,
			subapps: [None; 6],
			inside_subapp: false,
		}
	}
}

impl<'a, T: SubApp> App<'a, T> {
	fn display<D>(&'a self, target: &mut D)
	where
		D: DrawTarget<Color = BinaryColor>,
		<D as DrawTarget>::Error: std::fmt::Debug,
	{
		if self.inside_subapp
			&& let Some(subapp) = self.subapps[self.selection].as_ref()
		{
			target.clear(BinaryColor::Off).unwrap();
			subapp.display(target);
			return;
		}
		let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		Text::with_alignment(
			"Welcome to BeagleBadge!",
			Point::new(200, 20),
			text_style,
			Alignment::Center,
		)
		.draw(target)
		.unwrap();

		self.logos
			.iter()
			.enumerate()
			.try_for_each(|(i, logo)| logo.display(target, i == self.selection))
			.unwrap();
	}

	pub fn handle_events(&mut self, keycode: KeyCode) {
		if keycode == KeyCode::KEY_BACK && self.inside_subapp {
			self.inside_subapp = false;
		}
		if self.inside_subapp
			&& let Some(subapp) = self.subapps[self.selection].as_mut()
		{
			match keycode {
				KeyCode::KEY_UP => subapp.handle_events(JoystickEvents::Up),
				KeyCode::KEY_DOWN => subapp.handle_events(JoystickEvents::Down),
				KeyCode::KEY_LEFT => subapp.handle_events(JoystickEvents::Left),
				KeyCode::KEY_RIGHT => subapp.handle_events(JoystickEvents::Right),
				KeyCode::KEY_SELECT => subapp.handle_events(JoystickEvents::Select),
				_ => (),
			}
		} else {
			match keycode {
				KeyCode::KEY_RIGHT => {
					self.selection += 1;
					self.selection %= 6;
				}
				KeyCode::KEY_LEFT => {
					if self.selection == 0 {
						self.selection = 6;
					}
					self.selection -= 1;
				}
				KeyCode::KEY_UP | KeyCode::KEY_DOWN => {
					if self.selection < 3 {
						self.selection += 3;
					} else {
						self.selection -= 3;
					}
				}
				KeyCode::KEY_SELECT => {
					self.inside_subapp = true;
				}
				_ => (),
			}
		}
	}
}

struct Logo<'a> {
	position: Point,
	raw_image: ImageRaw<'a, BinaryColor, BigEndian>,
}

impl<'a> Logo<'a> {
	const PBM_DATA_OFFSET: usize = 53;
	const LOGO_OUTLINE_WIDTH: u32 = 4;
	const LOGO_SIZE: u32 = 80;

	fn from_pbm(position: Point, data: &'a [u8]) -> Logo<'a> {
		Logo {
			position,
			raw_image: ImageRaw::<BinaryColor>::new(
				&data[Self::PBM_DATA_OFFSET..],
				Self::LOGO_SIZE,
			),
		}
	}

	fn to_image(&'a self) -> Image<'a, ImageRaw<'a, BinaryColor, BigEndian>> {
		Image::new(&self.raw_image, self.position)
	}

	fn display<D>(&'a self, target: &mut D, draw_outline: bool) -> Result<(), D::Error>
	where
		D: DrawTarget<Color = BinaryColor>,
	{
		let style = PrimitiveStyle::with_fill(if draw_outline {
			BinaryColor::On
		} else {
			BinaryColor::Off
		});
		Rectangle::new(
			Point::new(
				self.position.x - Self::LOGO_OUTLINE_WIDTH as i32,
				self.position.y - Self::LOGO_OUTLINE_WIDTH as i32,
			),
			Size::new(
				Self::LOGO_SIZE + Self::LOGO_OUTLINE_WIDTH * 2,
				Self::LOGO_SIZE + Self::LOGO_OUTLINE_WIDTH * 2,
			),
		)
		.into_styled(style)
		.draw(target)?;
		self.to_image().draw(target)
	}
}

fn main() {
	let mut app = App::<swupdate::SWUpdate>::default();
	app.subapps[0] = Some(swupdate::SWUpdate);
	let fb = linuxfb::Framebuffer::new("/dev/fb0").unwrap();

	let mut data = fb.map().unwrap();

	let mut target = FramebufferTarget {
		data: &mut data,
		width: fb.get_size().0,
		height: fb.get_size().1,
		bpp: fb.get_bytes_per_pixel(),
	};

	target.clear(BinaryColor::Off);

	let mut device = Device::open("/dev/input/event0").unwrap();

	println!(
		"Listening for joystick events on: {}",
		device.name().unwrap_or("Unknown Device")
	);

	app.display(&mut target);

	loop {
		let events = device.fetch_events().expect("Failed to read events");

		for event in events {
			match event.destructure() {
				EventSummary::Key(_, keycode, 1) => app.handle_events(keycode),
				_ => (),
			}
		}
		app.display(&mut target);
	}
}
