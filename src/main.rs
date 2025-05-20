// Based on https://packt.medium.com/implementing-terminal-i-o-in-rust-4a44652b0f11

use clap::{Arg, ArgAction, Command};
use std::fs;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};

#[derive(Debug)]
struct Coordinates {
	pub x: usize,
	pub y: usize,
}
struct HexViewer {
	doc: Vec<u8>,
	doc_static: Vec<u8>,
	read_only: bool,
	history: Vec<(usize, u8)>,
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
		let doc = fs::read(file_name).unwrap();
		let size = termion::terminal_size().unwrap_or((80, 80));
		let hex_columns: usize = (size.0 as usize - 10) / 3;
		let rows = (doc.len() +hex_columns -1) / hex_columns;

		Self {
			doc: doc.clone(),
			doc_static: doc,
			read_only,
			history: vec![],
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
		let doc = vec![0; length];
		let size = termion::terminal_size().unwrap();
		let hex_columns: usize = (size.0 as usize - 10) / 3;
		let rows = (doc.len() +hex_columns -1) / hex_columns;

		Self {
			doc: doc.clone(),
			doc_static: doc,
			read_only,
			history: vec![],
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
			file_name: "untitled".to_string(),
		}
	}

	fn show_document(&mut self) {
		print!("{}{}", termion::clear::All,
			termion::cursor::Goto(1, 1));
		
		let mut buffer = String::new();
		for row in self.start_row..std::cmp::min(self.start_row + self.terminal_size.y - 3,self.rows)  {
			buffer.push_str(&format!("{:08X} |", row * self.hex_columns));
			for index in (row * self.hex_columns)..(row +1) * self.hex_columns {
				buffer.push_str(self.print_bit(index).as_str());
			}
			buffer.push_str("\r\n");
		}
		print!("{}", buffer);

		print!("{}", termion::cursor::Goto(1, (self.terminal_size.y - 1) as u16));

		if self.cur_byte >= 0 {
			print!(
				"{}{}{:08X} ({},{}) line-count={} Filename: {} {}{}",
				color::Fg(color::Red),
				style::Bold,
				self.cur_byte,
				(self.cur_pos.x - 9) / 3,
				self.cur_pos.y,
				self.rows,
				self.file_name,
				if self.read_only { "(read-only)" } else { "" },
				style::Reset
			);
		} else {
			print!(
				"{}{}-------- ({},{}) line-count={} Filename: {} {}{}",
				color::Fg(color::Red),
				style::Bold,
				(self.cur_pos.x - 9) / 3,
				self.cur_pos.y,
				self.rows,
				self.file_name,
				if self.read_only { "(read-only)" } else { "" },
				style::Reset
			);
		}

		// print!(
		// 	"{}",
		// 	termion::cursor::Goto(1, self.terminal_size.y as u16)
		// );

		self.set_pos(self.cur_pos.x, self.cur_pos.y);
	}

	fn print_bit(&self, index: usize) -> String {
		let mut buffer = String::new();

		if index >= self.doc.len() {
			if index == (self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3{
				buffer.push_str(&format!(" {}{}--{}",
				color::Bg(color::Red),
				color::Fg(color::White),
				style::Reset));
			} else {
				buffer.push_str(&format!(" {}--{}",
					color::Fg(color::Red),
					style::Reset));
			}
		} else {
			if index == self.cur_byte as usize {
				buffer.push_str(&format!(" {}{}{:02X}{}",
					color::Bg(color::Black),
					color::Fg(color::White),
					self.doc[index],
					style::Reset));
			} else {
				buffer.push_str(&format!(" {:02X}", self.doc[index]));
			}
		}

		return buffer;
	}

	fn set_pos(&mut self, x: usize, y: usize) {
		self.cur_pos.x = x;
		self.cur_pos.y = y;
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.len() { self.cur_byte = -1; }
		print!("{}",
			termion::cursor::Goto(self.cur_pos.x as u16, (self.cur_pos.y) as u16)
		);
	}

	fn run(&mut self) {
		let mut stdout = stdout().into_raw_mode().unwrap();
		let stdin = stdin();
		let mut keys = stdin.keys();
		while let Some(Ok(c)) = keys.next() {
			match c {
				Key::Ctrl('x') => {		// Exit
					if self.doc != self.doc_static {
						let prompt = "You have unsaved changes. Exit without saving? (Y/n): ";
						if let Some(answer) = self.read_input_line(prompt, "") {
							let answer = answer.trim().to_lowercase();
							if answer == "y" || answer == "yes" {
								break;
							} else {
								self.show_document();
								print!(
									"{}{}Exit canceled.{}",
									termion::cursor::Goto(1, self.terminal_size.y as u16),
									termion::clear::CurrentLine,
									style::Reset
								);
							}
						} else {
							self.show_document();
							print!(
								"{}{}Exit canceled.{}",
								termion::cursor::Goto(1, self.terminal_size.y as u16),
								termion::clear::CurrentLine,
								style::Reset
							);
						}
					} else {
						break;
					}
				}
				Key::Ctrl('o') => {		// Write out
					if !self.read_only {
						let default_name: String = if self.file_name == "untitled" { "".to_owned() } else { self.file_name.clone() };
						if let Some(path) = self.read_input_line("Output file path: ", &default_name) {
							if let Ok(mut file) = std::fs::File::create(path.as_str()) {
								match file.write_all(self.doc.by_ref()) {
									Ok(_) => {
										self.doc_static = self.doc.clone();
										self.file_name = path.clone();
										print!(
											"{}{}File saved to: {}{}",
											termion::cursor::Goto(1, self.terminal_size.y as u16),
											termion::clear::CurrentLine,
											path.as_str(),
											style::Reset
										);
									}
									Err(e) => {
										print!(
											"{}{}Error saving file: {}{}",
											termion::cursor::Goto(1, self.terminal_size.y as u16),
											termion::clear::CurrentLine,
											e,
											style::Reset
										);
									}
								}
							} else {
								print!(
									"{}{}Error opening file: {}{}",
									termion::cursor::Goto(1, self.terminal_size.y as u16),
									termion::clear::CurrentLine,
									path.as_str(),
									style::Reset
								);
							}
						} else {
							print!(
								"{}{}File saving canceled.{}",
								termion::cursor::Goto(1, self.terminal_size.y as u16),
								termion::clear::CurrentLine,
								style::Reset
							);
						}
					}
				}
				Key::Char('i') => {		// Insert Value
					if !self.read_only && self.cur_byte != -1 {
						let prompt = format!("New hex value for 0x{:08X}: ", self.cur_byte);
						if let Some(hex_input) = self.read_input_line(&prompt, &format!("{:02X}", self.doc[self.cur_byte as usize])) {
							if let Ok(value) = u8::from_str_radix(hex_input.trim(), 16) {
								if let Some(byte) = self.doc.get_mut(self.cur_byte as usize) {
									self.history.push((self.cur_byte as usize, *byte));
									*byte = value;
									self.show_document();
									print!(
										"{}{}Updated 0x{:08X}{}",
										termion::cursor::Goto(1, self.terminal_size.y as u16),
										termion::clear::CurrentLine,
										self.cur_byte,
										style::Reset
									);
								}
							} else {
								print!(
									"{}{}Invalid hexadecimal input!{}",
									termion::cursor::Goto(1, self.terminal_size.y as u16),
									termion::clear::CurrentLine,
									style::Reset
								);
							}
						} else {
							print!(
								"{}{}Changes Cancelled!{}",
								termion::cursor::Goto(1, self.terminal_size.y as u16),
								termion::clear::CurrentLine,
								style::Reset
							);
						}
					}
				}
				Key::Ctrl('z') => {		// Undo Changes
					if let Some(last) = self.history.pop() {
						if let Some(byte) = self.doc.get_mut(last.0) {
							*byte = last.1;
							self.show_document();
							print!(
								"{}{}Undid value of 0x{:08X}{}",
								termion::cursor::Goto(1, self.terminal_size.y as u16),
								termion::clear::CurrentLine,
								last.0,
								style::Reset
							);
						}
					}
				}

				// Navigation
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

	fn read_input_line(&mut self, prompt: &str, default_input: &str) -> Option<String> {
		let mut stdout = stdout();
		let mut input = String::from(default_input);

		let y = self.terminal_size.y as u16;
		print!("{}{}{}{}", termion::cursor::Goto(1, y), termion::clear::CurrentLine, prompt, input);
		stdout.flush().unwrap();

		for key in stdin().keys() {
			match key.unwrap() {
				Key::Esc | Key::Ctrl('c')=> return None, // Cancel input
				Key::Char('\n') => break, // Finish input
				Key::Char(c) => {
					input.push(c);
					print!("{}", c);
					stdout.flush().unwrap();
				}
				Key::Backspace => {
					if input.pop().is_some() {
						print!("{} {}", termion::cursor::Left(1), termion::cursor::Left(1));
						stdout.flush().unwrap();
					}
				}
				_ => {}
			}
		}

		self.show_document();

		if input.is_empty() { return None; }
		Some(input)
	}

	fn inc_x(&mut self) {
		if self.cur_pos.x < self.terminal_size.x -2 {
			self.cur_pos.x += 3;
		}
		self.cur_byte = ((self.cur_pos.y -1) * self.hex_columns + (self.cur_pos.x - 12) / 3) as isize;
		if self.cur_byte as usize >= self.doc.len() { self.cur_byte = -1; }
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
		if self.cur_byte as usize >= self.doc.len() { self.cur_byte = -1; }
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
		if self.cur_byte as usize >= self.doc.len() { self.cur_byte = -1; }

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
		if self.cur_byte as usize >= self.doc.len() { self.cur_byte = -1; }

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
                .conflicts_with_all(["input_pos", "input_flag", "read_only", "dump"]),
        )
        .arg(
            Arg::new("read_only")
                .short('r')
                .long("read-only")
                .help("Enable read-only mode")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["create", "dump"]),
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
					if index >= viewer.doc.len() {
						print!(" --");
					} else {
						print!(" {:02X}", viewer.doc[index]);
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
