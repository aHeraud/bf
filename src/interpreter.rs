use std::io::{stdin, Read};
use super::{Program, Instruction, Direction};

const MEMORY_SIZE: usize = 30000;

#[derive(Clone, Copy, Debug)]
pub enum InterpreterError {
	DataIndexOutOfBounds{ index: isize, array_size: usize },
	InvalidJump
}

pub struct Interpreter {
	program: Program,
	data_memory: Box<[u8]>,
	data_pointer: isize,
	instruction_pointer: isize
}

impl Interpreter {
	pub fn new(program: Program) -> Interpreter {
		Interpreter {
			program: program,
			data_memory: Box::new([0; MEMORY_SIZE]),
			data_pointer: 0,
			instruction_pointer: 0
		}
	}

	#[inline]
	fn get_data_index(&self) -> Result<usize, InterpreterError> {
		if self.data_pointer >= 0 && (self.data_pointer as usize) < self.data_memory.len() {
			Ok(self.data_pointer as usize)
		}
		else {
			Err(InterpreterError::DataIndexOutOfBounds{ index: self.data_pointer, array_size: self.data_memory.len() })
		}
	}

	#[must_use]
	pub fn step(&mut self) -> Result<bool, InterpreterError> {
		use self::Instruction::*;

		// TODO: should the data pointer wrap?
		if let Some(ref instruction) = self.program.instructions.get(self.instruction_pointer as usize) {
			match instruction {
				AddPointer(val) => {
					self.data_pointer += *val;
				},
				AddData(val) => {
					let data_index = self.get_data_index()?;
					self.data_memory[data_index] = (self.data_memory[data_index] as i8 as isize).wrapping_add(*val) as u8;
				},
				Input => {
					let stdin = stdin();
					let mut stdin_lock = stdin.lock();
					let mut buffer: [u8; 1] = [0; 1];
					let _ = stdin_lock.read(&mut buffer);
					self.data_memory[self.get_data_index()?] = buffer[0];
				},
				Output => {
					let index = self.get_data_index()?;
					print!("{}", self.data_memory[index] as char);
				},
				Jump{ id: _, target_id, direction } => {
					let cell_value = self.data_memory[self.get_data_index()?];
					if ((*direction == Direction::Forward) && cell_value == 0) || ((*direction == Direction::Backward) && cell_value != 0) {
						if let Some(index) = self.program.map.get(target_id) {
							self.instruction_pointer = *index as isize;
						}
						else {
							return Err(InterpreterError::InvalidJump);
						}
					}
				}
			}
			self.instruction_pointer += 1;

			Ok(false)
		}
		else {
			// end of program
			Ok(true)
		}
	}

	pub fn run(&mut self) -> Result<(), InterpreterError> {
		while !self.step()? {}
		Ok(())
	}
}