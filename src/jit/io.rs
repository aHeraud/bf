use libc::putchar;

pub extern "sysv64" fn output_char(c: usize) {
	unsafe {
		putchar(c as i32);
	}
}