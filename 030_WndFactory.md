```Rust
#![allow(dead_code)]

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
fn main() {
	let mut main_wnd_factory = WndFactory::<MainWnd>::new();
	let main_wnd = main_wnd_factory.yield_new_wnd();
	let hwnd = main_wnd.hwnd();

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
struct MainWnd {
	std_wnd: StdWnd,
}

impl MainWnd {
	fn hwnd(&self) -> HWND {
		return self.std_wnd.hwnd_std_wnd;
	}
}

impl WndFactoryCallee for MainWnd {
	fn new() -> MainWnd {
		MainWnd {
			std_wnd: StdWnd::new(),
		}
	}
}

impl StdWndCallee for MainWnd {
	fn pose_std_wnd(&mut self) -> &mut StdWnd {
		return &mut self.std_wnd;
	}
}

// -------------------------------------------------------------
struct CrtWndArgs {
	ex_style: u32,
	class_name: &'static [u16],
	window_title: &'static [u16],
	style: u32,
	left: i32,
	top: i32,
	width: i32,
	height: i32,
	hwnd_parent: HWND,
	hmenu: HMENU,
	hinstance: HINSTANCE,
	lp_param: *const core::ffi::c_void,
}

impl CrtWndArgs {
	fn new() -> Self {
		Self {
			ex_style: 0,
			class_name: StdWnd::TMP_WND_CLS_NAME,
			window_title: StdWnd::TMP_WND_TITLE,
			style: WS_OVERLAPPEDWINDOW,
			
			left: 100,
			top: 100,
			width: 500,
			height: 500,
			
			hwnd_parent: null!(),
			hmenu: null!(),
			hinstance: null!(),
			lp_param: null!(),
		}
	}
	
	fn call_create_window_ex(&self) -> HWND {
		unsafe {
			CreateWindowExW(
				self.ex_style,
				self.class_name.as_ptr(),
				self.window_title.as_ptr(),
				self.style,
				
				self.left,
				self.top,
				self.width,
				self.height,
				
				self.hwnd_parent,
				self.hmenu,
				self.hinstance,
				self.lp_param,
			)
		}
	}
}

// -------------------------------------------------------------
trait WndFactoryCallee {
	fn modify_wnd_class(_wnd_cls: &mut WNDCLASSW) {}
	fn wnd_cls_name() -> &'static [u16] {
		StdWnd::TMP_WND_CLS_NAME
	}	
	fn new() -> Self;
}

// -------------------------------------------------------------
trait StdWndCallee {
	fn pose_std_wnd(&mut self) -> &mut StdWnd;
	fn modify_crt_wnd_args(&self, _crt_wnd_args: &mut CrtWndArgs) {}
	fn on_crt_wnd_rsc(&self, _hwnd: HWND) {}
}

// -------------------------------------------------------------
struct WndFactory<T: WndFactoryCallee + StdWndCallee> {
	wnds: Vec<Box<T>>,
	wnd_cls_name: &'static [u16],
}

impl<T: WndFactoryCallee + StdWndCallee> WndFactory<T> {
	pub fn new() -> Self {
		unsafe {
			let mut wc = WNDCLASSW {
				style: 0,
				lpfnWndProc: Some(StdWnd::std_wnd_proc),
				cbClsExtra: 0,
				cbWndExtra: 0,
				hInstance: null!(),
				hIcon: LoadIconW(null!(), IDI_APPLICATION),
				hCursor: LoadCursorW(null!(), IDC_ARROW),
				hbrBackground: GetStockObject(WHITE_BRUSH) as HBRUSH,
				lpszMenuName: null!(),
				lpszClassName: StdWnd::TMP_WND_CLS_NAME.as_ptr(),
			};
			T::modify_wnd_class(&mut wc);
			
			if RegisterClassW(&wc) == 0 {
				panic!("!!! RegisterClassW(&wc) == 0");
			}

			Self {
				wnds: Vec::new(),
				wnd_cls_name: T::wnd_cls_name(),
			}
		}
	}

	pub fn yield_new_wnd<'a>(&'a mut self) -> &'a mut T {
		let mut new_wnd = Box::new(T::new());
		let mut crt_wnd_args = CrtWndArgs::new();
		new_wnd.modify_crt_wnd_args(&mut crt_wnd_args);
		
		let hwnd = crt_wnd_args.call_create_window_ex();
		if hwnd == null!() {
			panic!("!!! hwnd == null!()");
		}
		
		let std_wnd = new_wnd.pose_std_wnd();
		std_wnd.hwnd_std_wnd = hwnd;
		
		self.wnds.push(new_wnd);
		self.wnds.last_mut().unwrap().as_mut()
	}
}

// -------------------------------------------------------------
struct StdWnd {
	hwnd_std_wnd: HWND,
}

impl StdWnd {
	const TMP_WND_CLS_NAME: &'static [u16] = &u!("TMP_WND_CLS_NAME");
	const TMP_WND_TITLE: &'static [u16] = &u!("名称未設定");

	fn new() -> Self {
		Self {
			hwnd_std_wnd: null!(),
		}
	}

	unsafe extern "system" fn std_wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
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
}

```
