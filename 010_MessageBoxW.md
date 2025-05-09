## MessageBoxW
前回まで作っていたコードを以下のように書き換えてみましょう。

```
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn main() {
	let title = "テスト\0";
	let message = "こんにちは、世界！\0";
	unsafe {
		MessageBoxA(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK);
	}
}
```

前回まで、`let title = "TEST\0";` と書いていたところを `let title = "テスト\0";` などとしただけです。<br>
では、実行してみましょう。

```
> cargo run
```

プログラムは動きはしますが、表示される文字がおかしくなったと思います。

文字処理に関して、Rust では「8bit で１区切り」という考え方を標準としていますが、Windows では「16bit で１区切り」という考え方を標準としています。

---
```
use windows_sys::Win32::UI::WindowsAndMessaging::*;

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

fn main() {
	let title = u!("テスト");
	let message = u!("こんにちは、世界！");
	unsafe {
		MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK);
	}
}
```

続き -> [ウィンドウを開く](https://github.com/ki052020/rust-sample/blob/main/020_CreateWindow.md)
