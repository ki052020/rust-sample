```Rust
#![allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;

use windows_sys::{
	Win32::Foundation::*,
	Win32::UI::WindowsAndMessaging::*,
//	Win32::UI::Controls::*,
	Win32::Storage::FileSystem::*,
	Win32::System::ApplicationInstallationAndServicing::{ACTCTXW, CreateActCtxW, ActivateActCtx}
};

#[macro_use]
mod macros;

mod wnd_base;
use wnd_base::*;

// -------------------------------------------------------------
fn main() {
	// 透過型の layered child window を利用するために、マニフェストが必要となる
	load_manifest();

	let mut main_wnd_factory: WndFactory<MainWnd> = WndFactory::<MainWnd>::new();
	let main_wnd_rc: Rc<RefCell<MainWnd>> = main_wnd_factory.yield_new_wnd(&u!("テストウィンドウ"));

	{
		let wnd_base = &mut (main_wnd_rc.borrow_mut().wnd_base);
		wnd_base.show();
	}

	unsafe {
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
	wnd_base: WndBase,
	cnt: i32,
}

impl MainWnd {
	const WND_CLS_NAME: &'static [u16] = &u!("MAIN_WND_CLS");
		
	fn on_test_button(&mut self) {
		self.cnt += 1;
		println!("--- called -> on_test_button() / times -> {}", self.cnt);
	}
}

impl WndFactoryCallee for MainWnd {
	fn new(wnd_base: WndBase) -> MainWnd {
		MainWnd {
			wnd_base,
			cnt: 0,
		}
	}
	
	fn on_add_wb_items(main_wnd_rc: &Rc<RefCell<Self>>, wnd_base: &mut WndBase) {
		let wb_button = Box::new(WbButton::new(Rc::clone(main_wnd_rc), MainWnd::on_test_button, "テスト"));
		wnd_base.push_wb_item(wb_button);
	}

	fn wnd_cls_name() -> &'static [u16] {
		MainWnd::WND_CLS_NAME
	}
	
	fn wnd_base(&mut self) -> &mut WndBase {
		&mut self.wnd_base
	}

	fn on_crt_wnd_rsc(&mut self, hwnd: HWND) {
		TEST_create_child_wnd(hwnd);
	}
}

// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
// テストコード
#[allow(non_snake_case)]
fn TEST_create_child_wnd(hwnd_parent: HWND) {
	unsafe {
		let hwnd = CreateWindowExW(
			0, u!("EDIT").as_ptr(),
			u!("").as_ptr(),  // title
			WS_CHILD | WS_VISIBLE | WS_BORDER,
			10, 10,  // left, top
			300, 400, // width, height
			hwnd_parent,  // hWndParent
			null!(),  // hMenu
			null!(),  // hInstance
			null!()   // lpParam
		);
		
		if hwnd == null!() {
			panic!("!!! hwnd == null!()");
		}
	}
}
// <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<


// -------------------------------------------------------------
fn load_manifest() {
	// 透過型の layered child window を利用するために、以下のマニフェストが必要となる
	const MANIFEST_CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  	<compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1"> 
    	<application>
        	<!-- Windows 10 --> 
        	<supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    	</application>
  	</compatibility>
  	<dependency>
    	<dependentAssembly>
        	<assemblyIdentity
            type="win32"
            name="Microsoft.Windows.Common-Controls"
            version="6.0.0.0"
            processorArchitecture="*"
            publicKeyToken="6595b64144ccf1df"
            language="*"
        />
    	</dependentAssembly>
  	</dependency>
  	<application xmlns="urn:schemas-microsoft-com:asm.v3">
    	<windowsSettings>
      		<dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
    	</windowsSettings>
  	</application>
</assembly>
"#;

		// --------------------------------------------
	unsafe {
		let tmp_path_u16 = {
			let mut tmp_dir = [0u16; MAX_PATH as usize + 1];
			if GetTempPathW(tmp_dir.len() as u32, tmp_dir.as_mut_ptr()) == 0 {
				panic!("\n!!! failed -> GetTempPathW()\n");
			}

			let mut tmp_path = [0u16; MAX_PATH as usize + 1];
			if GetTempFileNameW(tmp_dir.as_ptr(), u!("tmp").as_ptr(), 0, tmp_path.as_mut_ptr()) == 0 {
				panic!("\n!!! failed -> GetTempFileNameW()\n");
			}
			tmp_path
		};
		
		let tmp_path_u8 = {
			let idx = tmp_path_u16.iter().position(|&x| x == 0).unwrap();
			String::from_utf16_lossy(&tmp_path_u16[..idx])
		};
		
		if let Err(err) = std::fs::write(&tmp_path_u8, MANIFEST_CONTENT) {
			panic!("\n!!! failed -> std::fs::write()\n   {err:?}\n");
		}
		
		const ACTCTX_FLAG_SET_PROCESS_DEFAULT: u32 = 0x010;
		let mut act_ctx = ACTCTXW {
			cbSize: std::mem::size_of::<ACTCTXW>() as u32,
			dwFlags: ACTCTX_FLAG_SET_PROCESS_DEFAULT,
			lpSource: tmp_path_u16.as_ptr(),
			wProcessorArchitecture: 0,
			wLangId: 0,
			lpAssemblyDirectory: null!(),
			lpResourceName: null!(),
			lpApplicationName: null!(),
			hModule: null!(),
		};
		
		let hctx = CreateActCtxW(&mut act_ctx);
		if hctx == INVALID_HANDLE_VALUE {
			panic!("\n!!! handle == INVALID_HANDLE_VALUE\n");
		}
		let mut activation_cookie: usize = 0;
		if ActivateActCtx(hctx, &mut activation_cookie) != TRUE {
			panic!("\n!!! ActivateActCtx(hctx, &mut activation_cookie) =! TRUE\n");
		}
		
		if let Err(err) = std::fs::remove_file(&tmp_path_u8) {
			panic!("\n!!! failed -> std::fs::remove_file()\n   {err:?}\n");
		}

		// --------------------------------------------
/*
		let icc = INITCOMMONCONTROLSEX {
			dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
			dwICC: ICC_STANDARD_CLASSES,
//			dwICC: 0,
		};
		if InitCommonControlsEx(&icc) == FALSE {
			panic!("\n!!! failed -> InitCommonControlsEx()\n");
		}
*/
	}
}
```

```Rust
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
//use std::ops::Deref;
use std::ops::DerefMut;
//use std::marker::PhantomData;

use windows_sys::{
	Win32::Foundation::*,
	Win32::UI::WindowsAndMessaging::*,
	Win32::Graphics::Gdi::*,
};

// -------------------------------------------------------------
pub struct CrtWndArgs {
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
	fn new<W: WndFactoryCallee>(wnd_title :&'static [u16]) -> Self {
		Self {
			ex_style: 0,
			class_name: W::wnd_cls_name(),
			window_title: wnd_title,
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
pub trait WndFactoryCallee {
	fn wnd_cls_name() -> &'static [u16];
	fn modify_wnd_class(_wnd_cls: &mut WNDCLASSW) {}

	// -----------------------------------
	fn new(wnd_base: WndBase) -> Self;
	fn on_add_wb_items(_new_wnd_rc: &Rc<RefCell<Self>>, _wnd_base: &mut WndBase) {}
	fn wnd_base(&mut self) -> &mut WndBase;
	fn modify_crt_wnd_args(&self, _crt_wnd_args: &mut CrtWndArgs) {}
	fn on_crt_wnd_rsc(&mut self, _hwnd: HWND) {}
}

// -------------------------------------------------------------
pub struct WndFactory<W: WndFactoryCallee> {
	wnds: Vec<Rc<RefCell<W>>>,
	wnd_cls_name: &'static [u16],
}

impl<W: WndFactoryCallee> WndFactory<W> {
	pub fn new() -> Self {
		unsafe {
			let mut wc = WNDCLASSW {
				style: 0,
				lpfnWndProc: Some(WndBase::wnd_proc),
				cbClsExtra: 0,
				cbWndExtra: 0,
				hInstance: null!(),
				hIcon: LoadIconW(null!(), IDI_APPLICATION),
				hCursor: LoadCursorW(null!(), IDC_ARROW),
				hbrBackground: GetStockObject(WHITE_BRUSH) as HBRUSH,
				lpszMenuName: null!(),
				lpszClassName: W::wnd_cls_name().as_ptr(),
			};
			W::modify_wnd_class(&mut wc);
			
			if RegisterClassW(&wc) == 0 {
				panic!("!!! RegisterClassW(&wc) == 0");
			}

			Self {
				wnds: Vec::new(),
				wnd_cls_name: W::wnd_cls_name(),
			}
		}
	}

	pub fn yield_new_wnd(&mut self, wnd_title :&'static [u16]) -> Rc<RefCell<W>> {
		self.wnds.push(Rc::new(RefCell::new(W::new(WndBase::new()))));
		
		let new_wnd_rc: Rc<RefCell<W>> = Rc::clone(self.wnds.last().unwrap());
		{
			let mut new_wnd: RefMut<'_, W> = new_wnd_rc.borrow_mut();
			let mut crt_wnd_args = CrtWndArgs::new::<W>(wnd_title);			
			new_wnd.modify_crt_wnd_args(&mut crt_wnd_args);

			let wnd_base: &mut WndBase = new_wnd.wnd_base();			
			W::on_add_wb_items(&new_wnd_rc, wnd_base);

			let hwnd_ol_wnd = crt_wnd_args.call_create_window_ex();
			if hwnd_ol_wnd == null!() {
				panic!("!!! hwnd_ol_wnd == null!()");
			}
			
			wnd_base.hwnd_wnd_base = hwnd_ol_wnd;
			unsafe {
				SetWindowLongPtrW(hwnd_ol_wnd, GWLP_USERDATA, (wnd_base as *mut WndBase) as isize);
			}
			
			// wb_item に ウィンドウリソース の生成を通知
			for wb_item in &mut wnd_base.wb_items {
				let ptr_boxed_wb_item = (wb_item as *const Box<dyn WbItem>) as isize;
				wb_item.on_crt_parent_wnd_rsc(ptr_boxed_wb_item, hwnd_ol_wnd);
			}

			// 最後に、OL Window に ウィンドウリソース の生成を通知する
			new_wnd.on_crt_wnd_rsc(hwnd_ol_wnd);
		}
		new_wnd_rc
	}
}

// -------------------------------------------------------------
pub trait WbItem {
	fn on_crt_parent_wnd_rsc(&mut self, ptr_boxed_wb_item: isize, hwnd_parent: HWND);
	
	fn on_click(&mut self) {}

	#[allow(non_snake_case)]
	fn DBG_stdout_DBG_description(&self);
}

impl dyn WbItem {
	// 将来修正するかも。本来は、Any を介すべき。
	fn downcast_from_box_into<T: WbItem>(self: &mut Box<Self>) -> &mut T {
		unsafe {
			let raw = Box::into_raw(Box::new(self));
			&mut *(raw as *mut T)
		}
	}
}

pub struct WndBase {
	hwnd_wnd_base: HWND,
	pub wb_items: Vec<Box<dyn WbItem>>,
}

impl WndBase {
	pub fn new() -> Self {
		Self {
			hwnd_wnd_base: null!(),
			wb_items: Vec::<Box<dyn WbItem>>::new(),
		}
	}

	pub fn hwnd(&self) -> HWND {
		self.hwnd_wnd_base
	}
	
	pub fn show(&self) {
		unsafe {
			ShowWindow(self.hwnd_wnd_base, SW_NORMAL);
		}
	}

	unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		unsafe {
			match msg {
				WM_COMMAND => {
					if w_param as i32 != ID_BUTTON {
						return 0;
					}
										
					let wb_button: &mut Box<dyn WbItem>
						= &mut *(GetWindowLongPtrW(l_param as HWND, GWLP_USERDATA) as *mut Box<dyn WbItem>);
					wb_button.on_click();
					return 0;
				},
				
				WM_DESTROY => {
					println!("detect -> WM_DESTROY");
					
					PostQuitMessage(0);
					return 0;
				},
				
				_ => return DefWindowProcW(hwnd, msg, w_param, l_param),
			};
		}
	}

	// -------------------------------------------------------------
	#[allow(non_snake_case)]
	pub fn yield_WbButton<T: 'static>(&mut self, tgt_obj: &Rc<RefCell<T>>, tgt_fn: fn(&mut T)) -> &mut WbButton<T> {
		let wb_button = Box::new(WbButton::<T> {
			tgt_obj: Rc::clone(tgt_obj),
			tgt_fn,
			DBG_description: None,
		});
				
		self.wb_items.push(wb_button);
		self.wb_items.last_mut().unwrap().downcast_from_box_into()
	}

	pub fn push_wb_item(&mut self, item: Box<dyn WbItem>) {
		self.wb_items.push(item);
	}
}

// -------------------------------------------------------------
// WbButton
const ID_BUTTON: i32 = 100;

#[allow(non_snake_case)]
pub struct WbButton<T> {
	tgt_obj: Rc<RefCell<T>>,
	tgt_fn: fn(&mut T),
	DBG_description: Option<String>,
}

impl<T> WbButton<T> {
	#[allow(non_snake_case)]
	pub fn new(tgt_obj: Rc<RefCell<T>>, tgt_fn: fn(&mut T), DBG_caption: &str) -> Self {
		Self {
			tgt_obj,
			tgt_fn,
			DBG_description: Some(String::from(DBG_caption)),
		}
	}
}

impl<T> WbItem for WbButton<T> {
	fn on_crt_parent_wnd_rsc(&mut self, ptr_boxed_wb_item: isize, hwnd_parent: HWND) {
		unsafe {
			let hwnd = CreateWindowExW(
				0,  // ex_style
				u!("BUTTON").as_ptr(),
				u!("テスト").as_ptr(),  // title
				WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
				320, 10,  // left, top
				100, 25, // width, height
				hwnd_parent,  // hWndParent
				ID_BUTTON as HMENU,
				null!(),  // hInstance
				null!()   // lpParam
			);
			
			if hwnd == null!() {
				panic!("!!! hwnd == null!()");
			}
			
			SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr_boxed_wb_item);
		}
	}

	fn on_click(&mut self) {
		let mut tgt_obj: RefMut<'_, T> = self.tgt_obj.as_ref().borrow_mut();
		(self.tgt_fn)(tgt_obj.deref_mut());
	}

	fn DBG_stdout_DBG_description(&self) {
		let desc = match &self.DBG_description {
			None => "None",
			Some(str) => &str,
		};
		println!("### WbButton::DBG_caption -> {desc}");
	}
}
```
