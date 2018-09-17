use std::vec::Vec;
use std::collections::HashMap;

use ::{Program, Instruction};

pub fn optimise(program: Program) -> Program {
	merge_instructions(program)
}

/// Merges similar neighbouring AddPointer/AddData instructions together.
/// For example, [AddData(1), AddData(-1), AddData(1)]
/// becomes [AddData(2)].
fn merge_instructions(mut program: Program) -> Program {
	use Instruction::*;

	let mut instructions: Vec<Instruction> = Vec::new();
	let mut map: HashMap<isize, usize> = HashMap::new();

	program.instructions.drain(..).for_each(|ins| {
		let last = instructions.pop();
		if let Some(last) = last {
			match (last, ins) {
				(AddPointer(a), AddPointer(b)) => {
					instructions.push(AddPointer(a + b));
				},
				(AddData(a), AddData(b)) => {
					instructions.push(AddData(a + b));
				},
				(_, Jump{ id, target_id, direction }) => {
					instructions.push(last);
					let index = instructions.len();
					instructions.push(Jump{ id, target_id, direction });
					map.insert(id, index);
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

	Program {
		instructions,
		map
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashMap;

	use ::{Instruction, Program, Direction};
	use super::*;

	#[test]
	fn merge_2_add_pointer_ins() {
		use Instruction::*;

		let program = Program {
			instructions: vec![AddPointer(1), AddPointer(1)],
			map: HashMap::new()
		};
		let result = merge_instructions(program);

		assert_eq!(result.map.len(), 0);
		assert_eq!(result.instructions, vec![AddPointer(2)]);
	}

	#[test]
	fn merge_3_add_data_ins() {
		use Instruction::*;

		let program = Program {
			instructions: vec![AddData(1), AddData(-1), AddData(1)],
			map: HashMap::new()
		};
		let result = merge_instructions(program);

		assert_eq!(result.map.len(), 0);
		assert_eq!(result.instructions, vec![AddData(1)]);
	}

	#[test]
	fn merge_relocate_jumps() {
		use Instruction::*;
		use Direction::*;
		
		let program = Program {
			instructions: vec![AddData(1), AddData(1), Jump{ id: 1, target_id: 2, direction: Forward }, AddPointer(1), AddPointer(1), Jump{ id: 2, target_id: 1, direction: Backward }],
			map: [(1, 2), (2, 5)].iter().cloned().collect()
		};
		let result = merge_instructions(program);

		assert_eq!(result.map, [(1, 1), (2, 3)].iter().cloned().collect());
		assert_eq!(result.instructions, vec![AddData(2), Jump{ id: 1, target_id: 2, direction: Forward }, AddPointer(2), Jump { id: 2, target_id: 1, direction: Backward }]);
	}
}