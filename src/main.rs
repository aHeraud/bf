#![feature(nll)]

use std::vec::Vec;
use std::collections::HashMap;

extern crate clap;
use clap::{Arg, App};

use std::fs::File;
use std::io::Read;

mod parser;
mod optimiser;
mod interpreter;

#[derive(Debug, Clone)]
pub struct Program {
	pub instructions: Vec<Instruction>,

	/// Maps an id to an index in the instruction vector (for jumps)
	pub map: HashMap<isize, usize>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
	Forward,
	Backward
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
	AddPointer(isize),
	AddData(isize),
	Input,
	Output,
	Jump{ id: isize, target_id: isize, direction: Direction }
}

pub fn main() {
	let matches = App::new("Brainfuck Interpreter")
		.version("0.1")
		.author("Achille Heraud <achille@heraud.xyz>")
		.about("A brainfuck interpreter")
		.arg(Arg::with_name("program")
			.takes_value(true)
			.value_name("FILE")
			.required(true)
			.empty_values(false))
		.get_matches();

	let program_path = matches.value_of("program").unwrap();

	let program = {
		let mut file = match File::open(&program_path) {
			Ok(file) => file,
			Err(e) => {
				println!("Failed to open {} - {}", program_path, e);
				return;
			}
		};

		let mut source = String::new();
		file.read_to_string(&mut source).expect("Failed to read program.");
		
		parser::parse(&source)
	}.unwrap();
	let program = optimiser::optimise(program);
	
	let mut interpreter = interpreter::Interpreter::new(program);
	interpreter.run().unwrap();
}