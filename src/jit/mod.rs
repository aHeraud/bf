use std::mem::transmute;
use std::borrow::BorrowMut;

use ::{Instruction, MEMORY_SIZE};

mod buffer;
mod translate;
mod io;

use self::translate::translate_to_native;
use self::buffer::Buffer;

pub fn execute(program: Vec<Instruction>) {
	let (native, mem) = set_up(program);
	let (native, mem) = run(native, mem);
	cleanup(native, mem);
}

fn set_up(program: Vec<Instruction>) -> (Buffer, Buffer) /* native_code, memory */ {
	let native = translate_to_native(program);
	unsafe {
		// we need to copy our program to a page that has been allocated with the execute flag, otherwise
		// we will not be able to actually execute it.
		let mut buffer = Buffer::allocate(native.len(), true);
		{
			let slice: &mut [u8] = buffer.borrow_mut();
			slice.copy_from_slice(&native);
		}

		// set up the data array that our program will use (non executable)
		let memory = Buffer::allocate(MEMORY_SIZE, false);
		
		(buffer, memory)
	}
}

fn run(native_code: Buffer, memory: Buffer) -> (Buffer, Buffer) {
	unsafe {
		let native_proc: (extern "sysv64" fn(usize) -> ()) = transmute(native_code.get_pointer());
		native_proc(memory.get_pointer() as usize);
	};
	(native_code, memory)
}

fn cleanup(native_code: Buffer, memory: Buffer) {
	unsafe {
		native_code.free();
		memory.free();
	}
}