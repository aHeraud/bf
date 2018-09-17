use std::vec::Vec;
use std::collections::HashMap;

use super::{Program, Instruction, Direction};

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

pub fn parse<'a>(program: &'a str) -> Result<Program, Vec<String>> {
	use self::Instruction::*;

	let mut instructions = Vec::new();
	let mut map = HashMap::new(); // maps jump ids to indexes in the instruction vector
	let mut stack: Vec<(isize, usize, ParsePosition)> = Vec::new();
	let mut id_counter = 1;

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
				let id = id_counter;
				instructions.push(Jump{ id: id, target_id: -1, direction: Direction::Forward });
				map.insert(id, index);
				stack.push((id, char_index, parse_position));
				id_counter += 1;
			},
			']' => {
				if let Some((jump_id, ..)) = stack.pop() {
					let index = instructions.len();
					let id = id_counter;

					// assign the correct target id to the matching '[' symbol.
					{
						let target_index = map.get(&jump_id).unwrap();
						let ref mut target = &mut instructions[*target_index];
						if let Jump{ id: _, ref mut target_id, direction: _ } = target {
							*target_id = id;
						}
					}

					map.insert(id, index);
					instructions.push(Jump{ id: id, target_id: jump_id, direction: Direction::Backward });
					id_counter += 1;
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

	if stack.len() > 0 {
		let mut errors = Vec::with_capacity(stack.len());
		while let Some((_, char_index, position)) = stack.pop() {
			errors.push(format!("'[' at line {}, col {} has no matching ']'.", position.line, char_index - parse_position.line_start));
		}
		return Err(errors);
	}

	Ok(Program {
		instructions,
		map
	})
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

		assert_eq!(program.instructions, expected_instructions);
		assert_eq!(program.map.len(), 0);
	}

	#[test]
	fn empty_loop() {
		use self::Instruction::*;
		use self::Direction::*;

		let source = "[]";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			Jump{ id: 1, target_id: 2, direction: Forward },
			Jump{ id: 2, target_id: 1, direction: Backward }
		];

		assert_eq!(program.instructions, expected_instructions);
		assert_eq!(program.map.get(&1), Some(&0));
		assert_eq!(program.map.get(&2), Some(&1));
	}

	#[test]
	fn empty_nested_loops() {
		use self::Instruction::*;
		use self::Direction::*;
		
		let source = "[[]]";
		let program = parse(source).unwrap();
		let expected_instructions = vec![
			Jump{ id: 1, target_id: 4, direction: Forward },
			Jump{ id: 2, target_id: 3, direction: Forward },
			Jump{ id: 3, target_id: 2, direction: Backward },
			Jump{ id: 4, target_id: 1, direction: Backward }
		];

		assert_eq!(program.instructions, expected_instructions);
		assert_eq!(program.map.get(&1), Some(&0));
		assert_eq!(program.map.get(&2), Some(&1));
		assert_eq!(program.map.get(&3), Some(&2));
		assert_eq!(program.map.get(&4), Some(&3));
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

		assert_eq!(program.instructions, expected_instructions);
		assert_eq!(program.map.len(), 0);
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