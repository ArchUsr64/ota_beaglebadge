use embedded_graphics::Drawable;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::{
	prelude::{Point, Size},
	primitives::Rectangle,
};

use crate::{FramebufferTarget, JoystickEvents, SCREEN_SIZE, SubApp};

const CELL_COUNT: (u32, u32) = (20, 15);

#[derive(Copy, Clone, Debug)]
enum Direction {
	Up,
	Down,
	Left,
	Right,
}

#[derive(Clone, Debug)]
pub struct Snake {
	moving_dir: Direction,
	snake: Vec<(i32, i32)>,
	apple: (u32, u32),
	game_over: bool,
}

impl Default for Snake {
	fn default() -> Self {
		Self {
			moving_dir: Direction::Right,
			snake: vec![(2, 7), (3, 7), (4, 7), (5, 7)],
			apple: (12, 7),
			game_over: false,
		}
	}
}

impl SubApp for Snake {
	fn handle_events(&mut self, event: crate::JoystickEvents) {
		let new_direction = match (self.moving_dir, event) {
			(Direction::Up, JoystickEvents::Up) => None,
			(Direction::Up, JoystickEvents::Left) => Some(Direction::Left),
			(Direction::Up, JoystickEvents::Right) => Some(Direction::Right),
			(Direction::Up, JoystickEvents::Down) => None,
			(Direction::Left, JoystickEvents::Up) => Some(Direction::Up),
			(Direction::Left, JoystickEvents::Left) => None,
			(Direction::Left, JoystickEvents::Right) => None,
			(Direction::Left, JoystickEvents::Down) => Some(Direction::Down),
			(Direction::Right, JoystickEvents::Up) => Some(Direction::Up),
			(Direction::Right, JoystickEvents::Left) => None,
			(Direction::Right, JoystickEvents::Right) => None,
			(Direction::Right, JoystickEvents::Down) => Some(Direction::Down),
			(Direction::Down, JoystickEvents::Up) => None,
			(Direction::Down, JoystickEvents::Left) => Some(Direction::Left),
			(Direction::Down, JoystickEvents::Right) => Some(Direction::Right),
			(Direction::Down, JoystickEvents::Down) => None,
			(_, JoystickEvents::Select) => {
				*self = Self::default();
				None
			}
		};
		if let Some(new_direction) = new_direction {
			self.moving_dir = new_direction;
		}
	}
	fn display(&self, target: &mut FramebufferTarget) {
		if self.game_over {
			let mut text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
			text_style.set_background_color(Some(BinaryColor::Off));

			Text::with_alignment(
				"Game Over!",
				Point::new(200, 120),
				text_style,
				Alignment::Center,
			)
			.draw(target)
			.unwrap();
			Text::with_alignment(
				"Press Select to restart",
				Point::new(200, 150),
				text_style,
				Alignment::Center,
			)
			.draw(target)
			.unwrap();
			return;
		}
		let rect_size = Size::new(SCREEN_SIZE.0 / CELL_COUNT.0, SCREEN_SIZE.1 / CELL_COUNT.1);
		for i in 0..CELL_COUNT.0 {
			for j in 0..CELL_COUNT.1 {
				let color = if self.snake.contains(&(i as i32, j as i32)) || self.apple == (i, j) {
					BinaryColor::On
				} else {
					BinaryColor::Off
				};
				Rectangle::new(
					Point::new((i * rect_size.width) as i32, (j * rect_size.height) as i32),
					rect_size,
				)
				.into_styled(PrimitiveStyle::with_fill(color))
				.draw(target);
			}
		}
	}
	fn update(&mut self) {
		let (delta_x, delta_y) = match self.moving_dir {
			Direction::Up => (0, -1),
			Direction::Down => (0, 1),
			Direction::Left => (-1, 0),
			Direction::Right => (1, 0),
		};
		let current_head = *self.snake.last().unwrap();
		let new_head = (current_head.0 + delta_x, current_head.1 + delta_y);
		if new_head.0 < 0
			|| new_head.0 >= CELL_COUNT.0 as i32
			|| new_head.1 < 0
			|| new_head.1 >= CELL_COUNT.1 as i32
			|| self.snake.contains(&new_head)
		{
			self.game_over = true;
		} else {
			self.snake.push(new_head);
			if new_head.0 == self.apple.0 as i32 && new_head.1 == self.apple.1 as i32 {
				loop {
					self.apple.0 = rand::random::<u32>() % CELL_COUNT.0;
					self.apple.1 = rand::random::<u32>() % CELL_COUNT.1;
					if !self
						.snake
						.contains(&(self.apple.0 as i32, self.apple.1 as i32))
					{
						break;
					}
				}
			} else {
				self.snake.remove(0);
			}
		}
	}
}
