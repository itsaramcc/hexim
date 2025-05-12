// Based on https://packt.medium.com/implementing-terminal-i-o-in-rust-4a44652b0f11

use clap::{Arg, ArgAction, Command};
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
	read_only: bool,
	cur_byte: isize,
	start_row: usize,
	rows: usize,
	hex_columns: usize,
	cur_pos: Coordinates,
	terminal_size: Coordinates,
	file_name: String,
}

impl HexViewer {
	fn init_file(file_name: &str, read_only: bool) -> Self {
		let doc_file = Doc { bytes: fs::read(file_name).unwrap() };
		let size = termion::terminal_size().unwrap();
		let hex_columns: usize = (size.0 as usize - 10) / 3;
		let rows = (doc_file.bytes.len() +hex_columns -1) / hex_columns;

		Self {
			doc: doc_file,
			read_only,
			cur_byte: 0,
			start_row: 0,
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

		fn init_length(length: usize, read_only: bool) -> Self {
		let doc_file = Doc { bytes: vec![0; length] };
		let size = termion::terminal_size().unwrap();
		let hex_columns: usize = (size.0 as usize - 10) / 3;
		let rows = (doc_file.bytes.len() +hex_columns -1) / hex_columns;

		Self {
			doc: doc_file,
			read_only,
			cur_byte: 0,
			start_row: 0,
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
			file_name: "untitled.txt".to_string(),
		}
	}

	fn show_document(&mut self) {
		print!("{}{}", termion::clear::All,
			termion::cursor::Goto(1, 1));
		
		for row in self.start_row..std::cmp::min(self.start_row + self.terminal_size.y - 3,self.rows)  {
			print!("{:08X} |", row * self.hex_columns);
			for index in (row * self.hex_columns)..(row +1) * self.hex_columns {
				self.print_bit(index);
			}
			println!("\r")
		}

		println!("{}", termion::cursor::Goto(1, (self.terminal_size.y - 2) as u16),);

		if self.cur_byte >= 0 {
			print!(
				"{}{}{:08X} ({},{}) line-count={} Filename: {}{}",
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
			print!(
				"{}{}-------- ({},{}) line-count={} Filename: {}{}",
				color::Fg(color::Red),
				style::Bold,
				(self.cur_pos.x - 9) / 3,
				self.cur_pos.y,
				self.rows,
				self.file_name,
				style::Reset
			);
		}

		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(1, self.terminal_size.y as u16)
		// );

		self.set_pos(self.cur_pos.x, self.cur_pos.y);
	}

	fn print_bit(&self, index: usize) {
		if index >= self.doc.bytes.len() {
			if index == (self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3{
				print!(" {}{}--{}",
				color::Bg(color::Red),
				color::Fg(color::White),
				style::Reset);
			} else {
				print!(" {}--{}",
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
		print!("{}",
			termion::cursor::Goto(self.cur_pos.x as u16, (self.cur_pos.y) as u16)
		);
	}

	fn run(&mut self) {
		let mut stdout = stdout().into_raw_mode().unwrap();
		let stdin = stdin();
		for c in stdin.keys() {
			match c.unwrap() {
				Key::Ctrl('q') => {
					break;
				}
				Key::Ctrl('o') => {
					if self.read_only {
						break;
					}
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
		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		// );
	}
	fn dec_x(&mut self) {
		if self.cur_pos.x > 12 {
			self.cur_pos.x -= 3;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }
		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		// );
	}
	fn inc_y(&mut self) {
		if self.cur_pos.y < self.rows {
			self.cur_pos.y += 1;
		}
		if self.cur_pos.y > self.start_row + self.terminal_size.y - 3 && self.start_row < self.rows - self.terminal_size.y + 3 { self.start_row += 1; }

		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }

		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		// );
	}
	fn dec_y(&mut self) {
		if self.cur_pos.y > 1 {
			self.cur_pos.y -= 1;
		}
		if self.cur_pos.y < self.start_row { self.start_row = self.cur_pos.y -1; }

		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.bytes.len() { self.cur_byte = -1; }

		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)
		// );
	}
}

fn main() {
	let matches = Command::new("hexim")
        .version("1.0")
        .about("A Hex Editor CLI written in Rust.")
		.arg(
            Arg::new("input_pos")
				.help("Input file (positional)")
                .index(1)
                .required(false)
				.conflicts_with_all(["input_flag", "create"]),
        )
        .arg(
            Arg::new("input_flag")
				.short('i')
				.long("input")
                .help("Input file (flag)")
                .required(false)
                .conflicts_with_all(["input_pos", "create"]),
        )
		.arg(
            Arg::new("create")
                .short('c')
                .long("create")
                .help("Create a new file with a specified length")
                .value_name("LENGTH")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with_all(["input_pos", "input_flag", "read_only"]),
        )
        .arg(
            Arg::new("read_only")
                .short('r')
                .long("read-only")
                .help("Enable read-only mode")
                .action(ArgAction::SetTrue)
                .conflicts_with("create"),
        )
		.arg(
            Arg::new("dump")
                .short('d')
                .long("dump")
                .help("Dumps the hex output into terminal")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["create", "read_only"]),
        )
        .get_matches();

    // Parse values
	 let input = matches
        .get_one::<String>("input_flag")
        .or(matches.get_one::<String>("input_pos"));
    let create = matches.get_one::<usize>("create");
	let read_only = matches.get_flag("read_only");
	let dump = matches.get_flag("dump");

    // Default behavior handling
    if create.is_none() && input.is_none() {
        eprintln!("Error: input file is required unless using --create");
		eprintln!("Usage: see --help for usage");
        std::process::exit(1);
    }

	// Handle Dump Flag
	if dump {
		if let Option::Some(file_name) = input {
			let viewer = HexViewer::init_file(file_name, read_only);
			for row in 0..viewer.rows {
				print!("{:08X} |", row * viewer.hex_columns);
				for index in (row * viewer.hex_columns)..(row +1) * viewer.hex_columns {
					if index >= viewer.doc.bytes.len() {
						print!(" {}--{}",
							color::Fg(color::Red),
							style::Reset);
					} else {
						print!(" {:02X}", viewer.doc.bytes[index]);
					}
				}
				println!()
			}
		}
		println!();
		std::process::exit(0);
	}
	
	// Open file & load into struct
	println!("{}", termion::screen::ToAlternateScreen);
	println!("{}", termion::cursor::Hide);
	if let Option::Some(file_name) = input {
		let mut viewer = HexViewer::init_file(file_name, read_only);
		viewer.show_document();
		viewer.run();
	}
	if let Option::Some(length) = create {
		let mut viewer = HexViewer::init_length(*length, read_only);
		viewer.show_document();
		viewer.run();
	}
	// Initialize viewer
	println!("{}", termion::cursor::Show);
	println!("{}", termion::screen::ToMainScreen);
}
