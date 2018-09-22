use std::vec::Vec;
use std::collections::HashMap;

use ::{ Instruction, Direction };

pub fn optimise(program: Vec<Instruction>) -> Vec<Instruction> {
	merge_instructions(program)
}

/// Merges similar neighbouring AddPointer/AddData instructions together.
/// For example, [AddData(1), AddData(-1), AddData(1)]
/// becomes [AddData(2)].
fn merge_instructions(mut program: Vec<Instruction>) -> Vec<Instruction> {
	use Instruction::*;

	let mut instructions: Vec<Instruction> = Vec::new();
	let mut jump_stack: Vec<usize> = Vec::new();

	program.drain(..).for_each(|ins| {
		let last = instructions.pop();
		if let Some(last) = last {
			match (last, ins) {
				(AddPointer(a), AddPointer(b)) => {
					instructions.push(AddPointer(a + b));
				},
				(AddData(a), AddData(b)) => {
					instructions.push(AddData(a.wrapping_add(b)));
				},
				(_, Jump{ index, direction }) => {
					instructions.push(last);
					let index = instructions.len();
					match direction {
						Direction::Forward => {
							instructions.push(Jump{ index: 0, direction });
							jump_stack.push(index);
						},
						Direction::Backward => {
							let current_index = instructions.len();
							let target_index = jump_stack.pop().unwrap();
							
							// update the target index of the forward jump
							let ref mut target = &mut instructions[target_index];
							if let Jump{ ref mut index, .. } = target {
								*index = current_index + 1;
							}

							instructions.push(Jump{ index: target_index + 1, direction });
						}
					}
				},
				_ => {
					instructions.push(last);
					instructions.push(ins);
				}
			}
		}
		else {
			instructions.push(ins);
		}
	});

	instructions
}

#[cfg(test)]
mod test {
	use ::{Instruction, Direction};
	use super::*;

	#[test]
	fn merge_2_add_pointer_ins() {
		use Instruction::*;

		let result = merge_instructions(vec![AddPointer(1), AddPointer(1)]);
		assert_eq!(result, vec![AddPointer(2)]);
	}

	#[test]
	fn merge_3_add_data_ins() {
		use Instruction::*;

		let result = merge_instructions(vec![AddData(1), AddData(-1), AddData(1)]);
		assert_eq!(result, vec![AddData(1)]);
	}

	#[test]
	fn merge_2_wrapping_add_data_ins() {
		use Instruction::*;
		let result = merge_instructions(vec![AddData(1), AddData(-1), AddData(-1)]);
		assert_eq!(result, vec![AddData(-1)]);
	}

	#[test]
	fn merge_relocate_jumps() {
		use Instruction::*;
		use Direction::*;
		
		let result = merge_instructions(vec![AddData(1), AddData(1), Jump{ index: 6, direction: Forward }, AddPointer(1), AddPointer(1), Jump{ index: 3, direction: Backward }]);
		assert_eq!(result, vec![AddData(2), Jump{ index: 4, direction: Forward }, AddPointer(2), Jump { index: 2, direction: Backward }]);
	}
}