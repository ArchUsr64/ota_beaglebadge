use embedded_graphics::{
	mono_font::{MonoTextStyle, ascii::FONT_10X20},
	pixelcolor::BinaryColor,
	prelude::*,
	primitives::{PrimitiveStyle, Rectangle},
	text::{Alignment, Text},
};

use crate::{FramebufferTarget, JoystickEvents, SubApp};

pub struct Specs {
	prev_cpu_total: u64,
	prev_cpu_idle: u64,
	cpu_usage: f32,
	ram_total: u64,
	ram_used: u64,
	ram_percentage: f32,
	cpu_temp: Option<f32>,
	cpu_freq: Option<f32>,
	uptime: u64,
}

fn parse_cpu_ticks(content: &str) -> Option<(u64, u64)> {
	let first_line = content.lines().next()?;
	if !first_line.starts_with("cpu ") {
		return None;
	}
	let parts: Vec<&str> = first_line.split_whitespace().skip(1).collect();
	let mut total: u64 = 0;
	let mut idle: u64 = 0;
	for (i, part) in parts.iter().enumerate() {
		if let Ok(val) = part.parse::<u64>() {
			total += val;
			if i == 3 || i == 4 {
				// idle and iowait
				idle += val;
			}
		}
	}
	Some((total, idle))
}

fn parse_meminfo_content(content: &str) -> Option<(u64, u64, f32)> {
	let mut mem_total_kb = None;
	let mut mem_avail_kb = None;
	let mut mem_free_kb = None;

	for line in content.lines() {
		let parts: Vec<&str> = line.split_whitespace().collect();
		if parts.len() >= 2 {
			if parts[0] == "MemTotal:" {
				mem_total_kb = parts[1].parse::<u64>().ok();
			} else if parts[0] == "MemAvailable:" {
				mem_avail_kb = parts[1].parse::<u64>().ok();
			} else if parts[0] == "MemFree:" {
				mem_free_kb = parts[1].parse::<u64>().ok();
			}
		}
	}

	let total = mem_total_kb?;
	let avail = mem_avail_kb.or(mem_free_kb)?;
	let used = total.saturating_sub(avail);

	let total_mb = total / 1024;
	let used_mb = used / 1024;
	let percentage = if total > 0 {
		(used as f32 / total as f32) * 100.0
	} else {
		0.0
	};

	Some((total_mb, used_mb, percentage))
}

fn parse_temp_content(content: &str) -> Option<f32> {
	if let Ok(millidegrees) = content.trim().parse::<i32>() {
		return Some(millidegrees as f32 / 1000.0);
	}
	None
}

fn parse_freq_content(content: &str) -> Option<f32> {
	if let Ok(freq_khz) = content.trim().parse::<u64>() {
		return Some(freq_khz as f32 / 1000.0);
	}
	None
}

fn parse_uptime_content(content: &str) -> Option<u64> {
	let first_part = content.split_whitespace().next()?;
	let seconds = first_part.parse::<f32>().ok()?;
	Some(seconds as u64)
}

fn read_cpu_ticks() -> Option<(u64, u64)> {
	let content = std::fs::read_to_string("/proc/stat").ok()?;
	parse_cpu_ticks(&content)
}

fn parse_meminfo() -> Option<(u64, u64, f32)> {
	let content = std::fs::read_to_string("/proc/meminfo").ok()?;
	parse_meminfo_content(&content)
}

fn read_cpu_temp() -> Option<f32> {
	for zone in 0..4 {
		let path = format!("/sys/class/thermal/thermal_zone{zone}/temp");
		if let Some(temp) = std::fs::read_to_string(&path)
			.ok()
			.and_then(|content| parse_temp_content(&content))
		{
			return Some(temp);
		}
	}
	None
}

fn read_cpu_freq() -> Option<f32> {
	let paths = [
		"/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq",
		"/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_cur_freq",
	];
	for path in &paths {
		if let Some(freq) = std::fs::read_to_string(path)
			.ok()
			.and_then(|content| parse_freq_content(&content))
		{
			return Some(freq);
		}
	}
	None
}

fn read_uptime() -> Option<u64> {
	let content = std::fs::read_to_string("/proc/uptime").ok()?;
	parse_uptime_content(&content)
}

impl Default for Specs {
	fn default() -> Self {
		let (cpu_total, cpu_idle) = read_cpu_ticks().unwrap_or((0, 0));
		let mut specs = Self {
			prev_cpu_total: cpu_total,
			prev_cpu_idle: cpu_idle,
			cpu_usage: 0.0,
			ram_total: 0,
			ram_used: 0,
			ram_percentage: 0.0,
			cpu_temp: None,
			cpu_freq: None,
			uptime: 0,
		};
		specs.update();
		specs
	}
}

impl SubApp for Specs {
	fn handle_events(&mut self, _event: JoystickEvents) {
		// No special handle needed.
	}

	fn update(&mut self) {
		if let Some((cpu_total, cpu_idle)) = read_cpu_ticks() {
			let delta_total = cpu_total.saturating_sub(self.prev_cpu_total);
			let delta_idle = cpu_idle.saturating_sub(self.prev_cpu_idle);
			self.prev_cpu_total = cpu_total;
			self.prev_cpu_idle = cpu_idle;
			if delta_total > 0 {
				self.cpu_usage =
					((1.0 - (delta_idle as f32 / delta_total as f32)) * 100.0).clamp(0.0, 100.0);
			}
		}

		if let Some((total, used, percentage)) = parse_meminfo() {
			self.ram_total = total;
			self.ram_used = used;
			self.ram_percentage = percentage;
		}

		self.cpu_temp = read_cpu_temp();
		self.cpu_freq = read_cpu_freq();

		if let Some(uptime) = read_uptime() {
			self.uptime = uptime;
		}
	}

	fn display(&self, target: &mut FramebufferTarget) {
		// Clear screen to white (since it's a sub-app, we want a clean canvas)
		target.clear(BinaryColor::Off).unwrap();

		// Draw border
		Rectangle::new(Point::new(5, 5), Size::new(390, 290))
			.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
			.draw(target)
			.unwrap();

		// Top title banner
		let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		Text::with_alignment(
			"SYSTEM STATISTICS",
			Point::new(200, 30),
			text_style,
			Alignment::Center,
		)
		.draw(target)
		.unwrap();

		// Horizontal line below title
		Rectangle::new(Point::new(15, 45), Size::new(370, 2))
			.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
			.draw(target)
			.unwrap();

		// Bottom horizontal line above footer
		Rectangle::new(Point::new(15, 250), Size::new(370, 2))
			.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
			.draw(target)
			.unwrap();

		// Vertical middle divider
		Rectangle::new(Point::new(200, 45), Size::new(2, 205))
			.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
			.draw(target)
			.unwrap();

		// LEFT COLUMN: CPU & System Info
		// 1. CPU Temp
		Text::new("Temp:", Point::new(15, 75), text_style)
			.draw(target)
			.unwrap();
		let temp_str = match self.cpu_temp {
			Some(t) => format!("{:.1} C", t),
			None => "N/A".to_string(),
		};
		Text::new(&temp_str, Point::new(95, 75), text_style)
			.draw(target)
			.unwrap();

		// 2. CPU Freq
		Text::new("Freq:", Point::new(15, 125), text_style)
			.draw(target)
			.unwrap();
		let freq_str = match self.cpu_freq {
			Some(f) => {
				if f >= 1000.0 {
					format!("{:.2} GHz", f / 1000.0)
				} else {
					format!("{:.0} MHz", f)
				}
			}
			None => "N/A".to_string(),
		};
		Text::new(&freq_str, Point::new(95, 125), text_style)
			.draw(target)
			.unwrap();

		// 3. Uptime
		Text::new("Uptime:", Point::new(15, 175), text_style)
			.draw(target)
			.unwrap();

		let days = self.uptime / 86400;
		let hours = (self.uptime % 86400) / 3600;
		let minutes = (self.uptime % 3600) / 60;
		let seconds = self.uptime % 60;

		let uptime_str = if days > 0 {
			format!("{}d {}h {}m", days, hours, minutes)
		} else if hours > 0 {
			format!("{}h {}m {}s", hours, minutes, seconds)
		} else if minutes > 0 {
			format!("{}m {}s", minutes, seconds)
		} else {
			format!("{}s", seconds)
		};
		Text::new(&uptime_str, Point::new(15, 210), text_style)
			.draw(target)
			.unwrap();

		// RIGHT COLUMN: Usage Gauges
		// 1. CPU Usage
		let cpu_label = format!("CPU Use: {:.1}%", self.cpu_usage);
		Text::new(&cpu_label, Point::new(215, 75), text_style)
			.draw(target)
			.unwrap();

		// CPU progress bar
		draw_progress_bar(target, 215, 95, 165, 16, self.cpu_usage);

		// 2. RAM Usage
		let ram_label = format!("RAM Use: {:.1}%", self.ram_percentage);
		Text::new(&ram_label, Point::new(215, 145), text_style)
			.draw(target)
			.unwrap();

		// RAM progress bar
		draw_progress_bar(target, 215, 165, 165, 16, self.ram_percentage);

		// RAM Detail
		let ram_detail = format!("{}/{} MB", self.ram_used, self.ram_total);
		Text::new(&ram_detail, Point::new(215, 205), text_style)
			.draw(target)
			.unwrap();

		// FOOTER: Back Hint
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
