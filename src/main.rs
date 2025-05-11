// Based on https://packt.medium.com/implementing-terminal-i-o-in-rust-4a44652b0f11

use std::env::args;
use std::fs;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

struct Doc {
	bytes: Vec<u8>
}

#[derive(Debug)]
struct Coordinates {
	pub x: usize,
	pub y: usize,
}
struct HexViewer {
	doc: Doc,
	cur_byte: isize,
	rows: usize,
	hex_columns: usize,
	cur_pos: Coordinates,
	terminal_size: Coordinates,
	file_name: String,
}

impl HexViewer {
	fn init(file_name: &str) -> Self {
		let doc_file = Doc { bytes: fs::read(file_name).unwrap() };
		let size = termion::terminal_size().unwrap();
		let hex_columns: usize = (size.0 as usize - 10) / 3;
		let rows = (doc_file.bytes.len() +hex_columns -1) / hex_columns;

		Self {
			doc: doc_file,
			cur_byte: 0,
			rows,
			hex_columns: hex_columns,
			cur_pos: Coordinates {
				x: 12,
				y: 1,
			},
			terminal_size: Coordinates {
				x: size.0 as usize,
				y: size.1 as usize,
			},
			file_name: file_name.into(),
		}
	}

	fn show_document(&mut self) {

		let pos = &self.cur_pos;
		let (old_x, old_y) = (pos.x, pos.y);

		print!("{}{}", termion::clear::All,
			termion::cursor::Goto(1, 1));
		
		if self.rows < self.terminal_size.y { 
			for row in 0..self.rows {
				print!("{:08X} |", row * self.hex_columns);
				for index in (row * self.hex_columns)..(row +1) * self.hex_columns {
					self.print_bit(index);
				}
				println!("\r")
			}
		} else {
			if pos.y <= self.terminal_size.y {
				for row in 0..(self.terminal_size.y -3) {
					print!("{:08X} |", row * self.hex_columns);
					for index in (row * self.hex_columns)..(row +1) * self.hex_columns {
						self.print_bit(index);
					}
					println!("\r");
				}
			} else {
				for row in (pos.y - (self.terminal_size.y -3))..pos.y {
					print!("{:08X} |", row * self.hex_columns);
					for index in (row * self.hex_columns)..(row +1) * self.hex_columns {
						self.print_bit(index);
					}
					println!("\r");
				}
			}
		}

		println!("{}", termion::cursor::Goto(0, (self.terminal_size.y - 2) as u16),);

		if self.cur_byte >= 0 {
			println!(
				"{}{} 0x{:08X} ({},{}) line-count={} Filename: {}{}",
				color::Fg(color::Red),
				style::Bold,
				self.cur_byte,
				(self.cur_pos.x - 9) / 3,
				self.cur_pos.y,
				self.rows,
				self.file_name,
				style::Reset
			);
		} else {
			println!(
				"{}{} UNDEFINED! ({},{}) line-count={} Filename: {}{}",
				color::Fg(color::Red),
				style::Bold,
				(self.cur_pos.x - 9) / 3,
				self.cur_pos.y,
				self.rows,
				self.file_name,
				style::Reset
			);
		}

		self.set_pos(old_x, old_y);
	}

	fn print_bit(&self, index: usize) {
		if index >= self.doc.bytes.len() {
			if index == (self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3{
				print!(" {}{}XX{}",
				color::Bg(color::Red),
				color::Fg(color::White),
				style::Reset);
			} else {
				print!(" {}XX{}",
					color::Fg(color::Red),
					style::Reset);
			}
		} else {
			if index == self.cur_byte as usize {
				print!(" {}{}{:02X}{}",
					color::Bg(color::Black),
					color::Fg(color::White),
					self.doc.bytes[index],
					style::Reset);
			} else {
				print!(" {:02X}", self.doc.bytes[index]);
			}
		}
	}

	fn set_pos(&mut self, x: usize, y: usize) {
		self.cur_pos.x = x;
		self.cur_pos.y = y;
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		println!("{}",
			termion::cursor::Goto(self.cur_pos.x as u16, (self.cur_pos.y) as u16)
		);
	}

	fn run(&mut self) {
		let mut stdout = stdout().into_raw_mode().unwrap();
		let stdin = stdin();
		for c in stdin.keys() {
			match c.unwrap() {
				Key::Ctrl('q') => {
					println!("{}", termion::cursor::Show);
					break;
				}
				Key::Left | Key::Char('h') => {
					self.dec_x();
					self.show_document();
				}
				Key::Right | Key::Char('l') => {
					self.inc_x();
					self.show_document();
				}
				Key::Up | Key::Char('k') => {
					self.dec_y();
					self.show_document();
				}
				Key::Down | Key::Char('j') => {
					self.inc_y();
					self.show_document();
				}
				Key::Backspace => {
					self.dec_x();
				}
				_=> {}
			}

			stdout.flush().unwrap();
		}
	}

	fn inc_x(&mut self) {
		if self.cur_pos.x < self.terminal_size.x -2 {
			self.cur_pos.x += 3;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		println!(
			"{}",
			termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		);
	}
	fn dec_x(&mut self) {
		if self.cur_pos.x > 12 {
			self.cur_pos.x -= 3;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		println!(
			"{}",
			termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		);
	}
	fn inc_y(&mut self) {
		if self.cur_pos.y < self.rows {
			self.cur_pos.y += 1;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		println!(
			"{}",
			termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		);
	}
	fn dec_y(&mut self) {
		if self.cur_pos.y > 1 {
			self.cur_pos.y -= 1;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		println!(
			"{}",
			termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		);
	}
}

fn main() {
	//Get arguments from command line
	let args: Vec<String> = args().collect();
	if args.len() < 2 {
		println!("Please provide file name as argument");
		std::process::exit(0);
	}
	//Check if file exists. If not, print error
	// message and exit process
	if !std::path::Path::new(&args[1]).exists() {
		println!("File does not exist");
		std::process::exit(0);
	}
	// Open file & load into struct
	println!("{}", termion::cursor::Hide);
	// Initialize viewer
	let mut viewer = HexViewer::init(&args[1]);
	viewer.show_document();
	viewer.run();
}
