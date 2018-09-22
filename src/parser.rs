use std::vec::Vec;
use std::collections::HashMap;

use super::{Instruction, Direction};

#[derive(Clone, Copy)]
struct ParsePosition {
	line_start: usize,
	line: usize
}

impl ParsePosition {
	pub fn new() -> ParsePosition {
		ParsePosition {
			/// The current line being parsed
			line: 0,

			/// The index in the source that the current line starts at
			line_start: 0 
		}
	}
}

pub fn parse<'a>(program: &'a str) -> Result<Vec<Instruction>, Vec<String>> {
	use self::Instruction::*;

	let mut instructions = Vec::new();
	let mut jump_stack: Vec<(usize, usize, ParsePosition)> = Vec::new();

	let mut parse_position = ParsePosition::new();
	for (char_index, c) in program.char_indices() {
		match c {
			'<' => instructions.push(AddPointer(-1)),
			'>' => instructions.push(AddPointer(1)),
			'+' => instructions.push(AddData(1)),
			'-' => instructions.push(AddData(-1)),
			',' => instructions.push(Input),
			'.' => instructions.push(Output),
			'[' => {
				let index = instructions.len();
				instructions.push(Jump{ index: 0, direction: Direction::Forward }); // index is set to 0 because we don't know the target yet
				jump_stack.push((index, char_index, parse_position));
			},
			']' => {
				if let Some((target_index, ..)) = jump_stack.pop() {
					instructions.push(Jump{ index: target_index + 1, direction: Direction::Backward }); // target_index points to the instruction after the '['
					// assign the correct index to the matching '[' symbol.
					{
						let current_index = instructions.len();
						let ref mut target = &mut instructions[target_index];
						if let Jump{ ref mut index, direction: _ } = target {
							*index = current_index;
						}
					}
				}
				else {
					return Err(vec![format!("']' at line {}, col {} has no matching '['.", parse_position.line, char_index - parse_position.line_start)]);
				}
			},
			'\n' => {
				parse_position.line += 1;
				parse_position.line_start = char_index;
			},
			_ => {}
		}
	}

	if jump_stack.len() > 0 {
		let mut errors = Vec::with_capacity(jump_stack.len());
		while let Some((_, char_index, position)) = jump_stack.pop() {
			errors.push(format!("'[' at line {}, col {} has no matching ']'.", position.line, char_index - parse_position.line_start));
		}
		return Err(errors);
	}

	Ok(instructions)
}

#[cfg(test)]
mod tests {
	use super::parse;
	use super::super::{Instruction, Direction};

	#[test]
	fn simple_program_no_loops() {
		use self::Instruction::*;

		let source = ">><<+-,.";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			AddPointer(1),
			AddPointer(1),
			AddPointer(-1),
			AddPointer(-1),
			AddData(1),
			AddData(-1),
			Input,
			Output
		];

		assert_eq!(program, expected_instructions);
	}

	#[test]
	fn empty_loop() {
		use self::Instruction::*;
		use self::Direction::*;

		let source = "[]";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			Jump{ index: 2, direction: Forward },
			Jump{ index: 1, direction: Backward }
		];

		assert_eq!(program, expected_instructions);
	}

	#[test]
	fn empty_nested_loops() {
		use self::Instruction::*;
		use self::Direction::*;
		
		let source = "[[]]";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			Jump{ index: 4, direction: Forward },
			Jump{ index: 3, direction: Forward },
			Jump{ index: 2, direction: Backward },
			Jump{ index: 1, direction: Backward }
		];

		assert_eq!(program, expected_instructions);
	}

	#[test]
	fn program_with_other_characters() {
		// Non instruction characters are valid and should just be ignored by the parser.
		use self::Instruction::*;

		let source = ">hello world<\n+";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			AddPointer(1),
			AddPointer(-1),
			AddData(1)
		];

		assert_eq!(program, expected_instructions);
	}

	#[test]
	fn unbalanced_forward_jump() {
		let source = "[[]";
		assert!(parse(source).is_err());
	}

	#[test]
	fn unbalanced_backwards_jump() {
		let source = "][]";
		assert!(parse(source).is_err());
	}
}