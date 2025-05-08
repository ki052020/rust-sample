// -------------------------------------------------------------
#[macro_export]
macro_rules! u {
	($str:literal) => {{
		const INPUT: &[u8] = $str.as_bytes();
		const OUTPUT_LEN: usize = windows_sys::core::utf16_len(INPUT) + 1;
		
		const fn output() -> [u16; OUTPUT_LEN] {
			let mut ret_ary = [0; OUTPUT_LEN];
			let mut idx_src = 0;
			let mut idx_dst = 0;
			while let Some((mut code, idx_src_new)) = windows_sys::core::decode_utf8_char(INPUT, idx_src) {
				idx_src = idx_src_new;
				if code <= 0xffff {
					ret_ary[idx_dst] = code as u16;
					idx_dst += 1;
				} else {
					code -= 0x10000;
					ret_ary[idx_dst] = 0xd800 + (code >> 10) as u16;
					ret_ary[idx_dst + 1] = 0xdc00 + (code & 0x3ff) as u16;
					idx_dst += 2;
				}
			}
			ret_ary
		}
		
		const OUTPUT: [u16; OUTPUT_LEN] = output();
		OUTPUT
	}};
}

// -------------------------------------------------------------
#[macro_export]
macro_rules! null {
	() => (std::ptr::null_mut())
}
