use std::cmp::min;
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
			rows,
			hex_columns: hex_columns,
			cur_pos: Coordinates {
				x: 1,
				y: rows,
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
		println!(
			"{}{}Welcome to HEXIM Hex Editor\r{}",
			color::Bg(color::Black),
			color::Fg(color::White),
			style::Reset
		);

		for row in 0..self.rows {
			print!("{:08X} |", row * self.hex_columns);
			for index in (row * self.hex_columns)..min(((row +1) * self.hex_columns), self.doc.bytes.len()) {
				print!(" {:02X}", self.doc.bytes[index])
			}
			println!("\r")
		}

		println!("{}", termion::cursor::Goto(0, (self.terminal_size.y - 2) as u16),);

		println!(
			"{}{} line-count={} Filename: {}{}",
			color::Fg(color::Red),
			style::Bold,
			self.rows,
			self.file_name,
			style::Reset
		);
		self.set_pos(old_x, old_y);
	}

	fn set_pos(&mut self, x: usize, y: usize) {
		self.cur_pos.x = x;
		self.cur_pos.y = y;
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
					break;
			}
			_=> {}
			}
			stdout.flush().unwrap();
		}
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
	println!("{}", termion::cursor::Show);
	// Initialize viewer
	let mut viewer = HexViewer::init(&args[1]);
	viewer.show_document();
	viewer.run();
}
