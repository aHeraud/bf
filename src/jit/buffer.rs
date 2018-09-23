use std::os::raw;
use std::borrow::{Borrow, BorrowMut};
use std::slice;

#[cfg(not(target_os="windows"))]
use libc;

#[cfg(target_os="windows")]
use winapi;

const PAGE_SIZE: usize = 0x1000;

pub struct Buffer {
	pointer: *mut raw::c_void,
	size: usize
}

impl Buffer {
	#[cfg(not(target_os="windows"))]
	pub unsafe fn allocate(size: usize) -> Buffer {
		let pointer = libc::memalign(PAGE_SIZE, size);
		libc::mprotect(pointer, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
		Buffer {
			pointer: pointer as *mut raw::c_void,
			size
		}
	}

	#[cfg(target_os="windows")]
	pub unsafe fn allocate(size: usize) -> Buffer {
		use winapi::um::memoryapi::VirtualAlloc;
		use winapi::um::winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};
		use std::ptr::null_mut;
		let pointer = VirtualAlloc(null_mut(), size, MEM_COMMIT, PAGE_EXECUTE_READWRITE);
		Buffer {
			pointer,
			size
		}
	}

	#[cfg(not(target_os="windows"))]
	pub unsafe fn free(self) {
		libc::free(self.pointer as *mut libc::c_void);
	}

	#[cfg(target_os="windows")]
	pub unsafe fn free(self) {
		use winapi::um::memoryapi::VirtualFree;
		use winapi::um::winnt::MEM_RELEASE;
		VirtualFree(self.pointer, 0, MEM_RELEASE);
	}

	pub fn len(&self) -> usize {
		self.size
	}

	pub unsafe fn get_pointer(&self) -> *mut raw::c_void {
		self.pointer
	}
}

impl Borrow<[u8]> for Buffer {
	fn borrow(&self) -> &[u8] {
		unsafe {
			slice::from_raw_parts(self.pointer as *const u8, self.size)
		}
	}
}

impl BorrowMut<[u8]> for Buffer {
	fn borrow_mut(&mut self) -> &mut [u8] {
		unsafe {
			slice::from_raw_parts_mut(self.pointer as *mut u8, self.size)
		}
	}
}