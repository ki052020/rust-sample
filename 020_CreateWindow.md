```
[dependencies.windows-sys]
version = "*"
features = [
	"Win32_Foundation",
	"Win32_UI_WindowsAndMessaging",
	"Win32_Graphics_Gdi",
]
```

```Rust
use windows_sys::{
	Win32::Foundation::*,
	Win32::UI::WindowsAndMessaging::*,
	Win32::Graphics::Gdi::*,
};

// -------------------------------------------------------------
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
macro_rules! null {
	() => (std::ptr::null_mut())
}

// -------------------------------------------------------------
const WND_CLS_NAME: &'static [u16] = &u!("test_wnd_class");

fn main() {
	regist_wnd_class();
	let hwnd = create_wnd();

	unsafe {
		ShowWindow(hwnd, SW_NORMAL);
		let mut msg = std::mem::zeroed::<MSG>();
		loop {
			if GetMessageW(&mut msg, null!(), 0, 0) == 0 {
				return;
			}
			TranslateMessage(&mut msg);
			DispatchMessageW(&mut msg);
		}
	}
}

// -------------------------------------------------------------
fn create_wnd() -> HWND {
	unsafe {
		let hwnd = CreateWindowExW(
			0,
			WND_CLS_NAME.as_ptr(),
			u!("テスト ウィンドウ").as_ptr(),
			WS_OVERLAPPEDWINDOW,
			100, 100,  // left, top
			500, 500,  // width, height
			null!(),  // hWndParent
			null!(),  // hMenu
			null!(),  // hInstance
			null!()   // lpParam
		);
		
		if hwnd == null!() {
			panic!("!!! hwnd == null!()");
		}
		hwnd
	}
}

// -------------------------------------------------------------
fn regist_wnd_class() {
	unsafe {
		let mut wc = std::mem::zeroed::<WNDCLASSW>();
		
		wc.lpfnWndProc = Some(wnd_proc);
		wc.hIcon = LoadIconW(null!(), IDI_APPLICATION);
		wc.hCursor = LoadCursorW(null!(), IDC_ARROW);
		wc.hbrBackground = GetStockObject(WHITE_BRUSH) as HBRUSH;
		wc.lpszClassName = WND_CLS_NAME.as_ptr();
		
		if RegisterClassW(&wc) == 0 {
			panic!("!!! RegisterClassW(&wc) == 0");
		}
	}
}

// -------------------------------------------------------------
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	unsafe {
		match msg {
			WM_DESTROY => {
				PostQuitMessage(0);
				return 0;
			},
			_ => return DefWindowProcW(hwnd, msg, w_param, l_param),
		};
	}
}
```
