#![feature(nll)]

extern crate clap;
extern crate libc;

#[cfg(target_os="windows")]
extern crate winapi;

use clap::{Arg, App};

use std::fs::File;
use std::io::Read;

mod parser;
mod optimiser;
mod interpreter;

#[cfg(target_arch="x86_64")]
mod jit;

const MEMORY_SIZE: usize = 30000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
	Forward,
	Backward
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
	AddPointer(isize),
	AddData(i8),
	Input,
	Output,
	Jump{ index: usize, direction: Direction }
}

pub fn main() {
	let matches = App::new("Brainfuck Interpreter")
		.version("0.1")
		.author("Achille Heraud <achille@heraud.xyz>")
		.about("A brainfuck interpreter")
		.arg(Arg::with_name("interpret")
			.short("i")
			.takes_value(false)
			.help("use the interpreter")
			.required(false))
		.arg(Arg::with_name("program")
			.takes_value(true)
			.value_name("FILE")
			.required(true)
			.empty_values(false))
		.get_matches();

	let program_path = matches.value_of("program").unwrap();
	let use_interpreter = matches.is_present("interpret");

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
	
	if use_interpreter {
		let mut interpreter = interpreter::Interpreter::new(program);
		interpreter.run().unwrap();
	}
	else {
		#[cfg(target_arch="x86_64")]
		jit::execute(program);

		#[cfg(not(target_arch="x86_64"))]
		println!("Dynarec only implemented for x86_64, use the interpreter instead by passing the -i flag.");
	}
}
