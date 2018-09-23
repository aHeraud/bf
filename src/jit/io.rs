use libc::{putchar, getchar};

pub extern "sysv64" fn input_char() -> i32 {
	unsafe {
		getchar()
	}
}

pub extern "sysv64" fn output_char(c: usize) {
	unsafe {
		putchar(c as i32);
	}
}