use std::vec::Vec;
use std::mem::transmute;
use std::os::raw::c_void;

use ::Instruction;
use jit::io::{input_char, output_char};

/// Layout for x86/x86_64
/// Data Pointer: EAX/RAX
/// Temp Values: EBX/RCX
#[cfg(target_arch="x86_64")]
pub fn translate_to_native(program: Vec<Instruction>) -> Vec<u8> {
	use ::Instruction::*;
	use ::Direction;

	let mut instructions: Vec<u8> = Vec::new();
	let mut jump_stack: Vec<(usize, usize)> = Vec::new(); // (address_index /* address is a signed 4-byte offset */, next_instruction_index)

	// The translated subroutine takes the address of the memory array as the first argument, which is placed by the called into RDI
	// Clobbers RAX
	// entry:
	// PUSH RBP
	// MOV RBP, RSP
	// SUB RSP, 24 // even though we are only using 16 bytes, this needs to be 16 byte aligned to match the sysv64 calling convention
	// PUSH RCX
	// MOV RAX, RDI
	instructions.extend(vec![0x55, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x18, 0x51, 0x48, 0x89, 0xF8]);

	// set up pointers for io:
	//     RBP[-8] = output_char
	//     RBP[-16] = input_char
	// assembly:
	//     MOVABSQ RCX, output_char
	//     MOV [RBP-8], RCX
	//     MOVABSQ RCX, input_char
	//     MOV [RBP-16], RCX
	instructions.extend(vec![0x48, 0xB9]); // MOVABSQ RCX, ..
	{
		let proc_ptr: usize = unsafe { transmute(output_char as *const c_void) };
		let immediate_value: [u8; 8] = unsafe { transmute(proc_ptr) };
		instructions.extend(immediate_value.iter()); // .., output_char
	}
	instructions.extend(vec![0x48, 0x89, 0x4D, 0xF8]); // MOV QWORD PTR [RBP - 8], RCX
	instructions.extend(vec![0x48, 0xB9]); // MOVABSQ RCX, ..
	{
		let proc_ptr: usize = unsafe { transmute(input_char as *const c_void) };
		let immediate_value: [u8; 8] = unsafe { transmute(proc_ptr) };
		instructions.extend(immediate_value.iter());
	}
	instructions.extend(vec![0x48, 0x89, 0x4D, 0xF0]); // MOV QWORD PTR [RBP - 8], RCX

	program.iter().for_each(|ins| {
		match ins {
			AddData(val) => {
				// ADD BYTE PTR [RAX], $val
				instructions.extend(vec![0x80, 0x00, *val as u8]);
			},
			AddPointer(val) => {
				// MOVABSQ RCX, $val  ; load 64-bit immediate
				// ADD RAX, RCX
				let immediate_value: [u8; 8] = unsafe { transmute(*val) };
				instructions.extend(vec![0x48, 0xB9]); // MOVABSQ RCX, ..
				instructions.extend(immediate_value.iter()); // .., $val
				instructions.extend(vec![0x48, 0x01, 0xC8]) // ADD RAX, RCX
			},
			Input => {
				// PUSH RAX
				// CALL [RBP - 16]
				// MOV RCX, RAX
				// POP RAX
				// MOV BYTE PTR [RAX], CL
				
				instructions.extend(vec![0x50]); // PUSH RAX
				instructions.extend(vec![0xFF, 0x55, 0xF0]); // CALL [RBP-16]
				instructions.extend(vec![0x48, 0x89, 0xC1]); // MOV RCX, RAX
				instructions.extend(vec![0x58]); // POP RAX
				instructions.extend(vec![0x88, 0x08]) // MOV BYTE PTR [RAX], CL
			},
			Output => {
				// MOVZX RDI, BYTE PTR [RAX]
				// POP RCX
				// PUSH RAX
				// CALL [RBP - 8]
				// POP RAX
				// PUSH RCX

				instructions.extend(vec![0x48, 0x0F, 0xB6, 0x38]); // MOVZX RDI, BYTE PTR [RAX]
				instructions.extend(vec![0x59]); // POP RCX
				instructions.extend(vec![0x50]); // PUSH RAX
				instructions.extend(vec![0xFF, 0x55, 0xF8]); // CALL [RBP - 8]
				instructions.extend(vec![0x58]); // POP RAX
				instructions.extend(vec![0x51]); // PUSH RCX
			},
			Jump{ index: _, direction } => {
				match direction {
					Direction::Forward => {
						// CMP BYTE PTR [RAX], 0
						// JZ $target
						// jump is a relative jump, so we don't need to know the address, just an offset from the next instruction
						instructions.extend(vec![0x80, 0x38, 0x00]); // CMP BYTE PTR [RAX], 0
						let address_index = instructions.len() + 2;
						instructions.extend(vec![0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // jump target still unknown, so for now it's set to 0
						let next_instruction_index = instructions.len();
						jump_stack.push((address_index, next_instruction_index));
					},
					Direction::Backward => {
						// CMP BYTE PTR [RAX], 0
						// JNZ $target
						// jump is a relative jump, so we don't need to know the address, just an offset from the next instruction
						let (address_index, target) = jump_stack.pop().expect("invalid program: mismatched jumps (']' missing '[')");
						instructions.extend(vec![0x80, 0x38, 0x00]); // CMP BYTE PTR [RAX], 0
						instructions.extend(vec![0x0F, 0x85]); // JNZ 
						let next_offset = (instructions.len() + 4) as isize; // offset of next instruction after the jump
						let rel32 = ((target as isize) - next_offset) as i32;
						let rel32_bytes: [u8; 4] = unsafe { transmute(rel32) };
						instructions.extend(rel32_bytes.iter());

						//now modify the offset of the matching forward jump
						let rel32 = (next_offset - (target) as isize) as i32;
						let rel32_bytes: [u8; 4] = unsafe { transmute(rel32) };
						&mut instructions[address_index..address_index+4].clone_from_slice(&rel32_bytes);
					}
				}
			},

		}
	});

	// cleanup:
	// POP RCX
	// MOV RSP, RBP ; clean up local variables
	// POP RBP ; restore stack base pointer
	// RET
	instructions.extend(vec![0x59, 0x48, 0x89, 0xEC, 0x5D, 0xC3]);

	instructions
}