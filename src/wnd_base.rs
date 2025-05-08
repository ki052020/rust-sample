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
// null 終端の utf16 文字列を生成する
pub fn str_to_u16(str: &str) -> Vec<u16> {
	unsafe {
		let bytes_src: &[u8] = str.as_bytes();
		let len_u16_with_null = windows_sys::core::utf16_len(bytes_src) + 1;
		let mut ret_vec = Vec::<u16>::with_capacity(len_u16_with_null);
		ret_vec.set_len(len_u16_with_null);
		
		let mut ptr_dst = ret_vec.as_mut_ptr();
		let mut idx_src = 0;
		while let Some((mut code, idx_src_new)) = windows_sys::core::decode_utf8_char(bytes_src, idx_src) {
			idx_src = idx_src_new;

			if code <= 0xffff {
				*ptr_dst = code as u16;
				ptr_dst = ptr_dst.add(1);
			} else {
				code -= 0x10000;
				*ptr_dst = 0xd800 + (code >> 10) as u16;
				ptr_dst = ptr_dst.add(1);
				*ptr_dst = 0xdc00 + (code & 0x3ff) as u16;
				ptr_dst = ptr_dst.add(1);
			}
		}
		*ptr_dst = 0;  // 終端文字
		
		ret_vec
	}
}

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
	fn new() -> Self;
	fn ntfy_add_wb_items(_new_wnd_rc: &Rc<RefCell<Self>>, _wnd_base: &mut WndBase) {}
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
	
		self.wnds.push(Rc::new(RefCell::new(W::new())));
		let new_wnd_rc: Rc<RefCell<W>> = Rc::clone(self.wnds.last().unwrap());
		
		unsafe {
			let mut new_wnd: RefMut<'_, W> = new_wnd_rc.borrow_mut();
			let mut crt_wnd_args = CrtWndArgs::new::<W>(wnd_title);			
			new_wnd.modify_crt_wnd_args(&mut crt_wnd_args);

			let wnd_base: &mut WndBase = new_wnd.wnd_base();			
			W::ntfy_add_wb_items(&new_wnd_rc, wnd_base);

			let hwnd_ol_wnd: HWND = crt_wnd_args.call_create_window_ex();
			if hwnd_ol_wnd == null!() {
				panic!("!!! hwnd_ol_wnd == null!()");
			}
			
			wnd_base.hwnd_wnd_base = hwnd_ol_wnd;
			SetWindowLongPtrW(hwnd_ol_wnd, GWLP_USERDATA, (wnd_base as *mut WndBase) as isize);
			
			// wb_item に ウィンドウリソース の生成を通知
			for wb_item in &mut wnd_base.wb_items {
				let ptr_boxed_wb_item = (wb_item as *mut Box<dyn WbItem>) as isize;
				wb_item.on_crt_parent_wnd_rsc(ptr_boxed_wb_item, hwnd_ol_wnd);
/*
				let ptr_rc_wb_item = (wb_item as *const Rc<dyn WbItem>) as isize;
				wb_item.on_crt_parent_wnd_rsc(ptr_rc_wb_item, hwnd_ol_wnd);
*/
			}

			// 最後に、OL Window に ウィンドウリソース の生成を通知する
			new_wnd.on_crt_wnd_rsc(hwnd_ol_wnd);
		}
		new_wnd_rc
	}
}

////////////////////////////////////////////////////////////////
// WbItemBase
#[allow(non_snake_case)]
pub struct WbItemBase {
	// 将来、x, y はグリッドから算出されるようにするため Option<> にしている
	x: Option<i32>,
	y: Option<i32>,
	
	// Panel を考えて、値は Option<> にしている
	width: Option<i32>,
	height: Option<i32>,
	
	DBG_description: String,
}

impl WbItemBase {
	fn with_xy_size(x: i32, y: i32, width: i32, height: i32) -> Self {
		Self {
			x: Some(x),
			y: Some(y),
			
			width: Some(width),
			height: Some(height),
			
			DBG_description: String::from("None"),
		}
	}
	
	pub fn set_width(&mut self, width: i32) -> &mut Self {
		self.width = Some(width);
		self
	}
	
	#[allow(non_snake_case)]
	pub fn set_DBG_description(&mut self, description: &str) -> &mut Self {
		self.DBG_description = String::from(description);
		self
	}
}

////////////////////////////////////////////////////////////////
// トレイト: WbItem
pub trait WbItem {
	fn wb_item_base(&mut self) -> &mut WbItemBase;
	fn on_crt_parent_wnd_rsc(&mut self, ptr_rc_wb_item: isize, hwnd_parent: HWND);

	// wnd_proc からコールされるため、ジェネリクスが利用できない
	// そのため、trait オブジェクトとしてインターフェイスを用意することにした
	fn on_click(&mut self) {}

	#[allow(non_snake_case)]
	fn DBG_stdout_DBG_description(&mut self) {
		println!("### DBG_description -> {}", self.wb_item_base().DBG_description);
	}
}

impl dyn WbItem {
	// 将来修正するかも。本来は、Any を介すべき。
	fn leak_from_box<'a, T>(self: &'a mut Box<dyn WbItem>) -> &'a mut T {
		unsafe {
			&mut *((self.as_mut() as *mut dyn WbItem) as *mut T)
		}
	}

	// 将来修正するかも。本来は、Any を介すべき。
	fn leak_from_rc<'a, T>(self: Rc<dyn WbItem>) -> &'a mut T {
		unsafe {
			&mut *((Rc::into_raw(self) as *mut dyn WbItem) as *mut T)
		}
	}
}


////////////////////////////////////////////////////////////////
// WndBase
pub struct WndBase {
	hwnd_wnd_base: HWND,
	pub wb_items: Vec<Box<dyn WbItem>>,
//	pub wb_items: Vec<Rc<dyn WbItem>>,
}

impl WndBase {
	const ID_BUTTON: i32 = 100;
	const ID_TBOX: i32 = 101;
	
	// -------------------------------------------------------------
	pub fn new() -> Self {
		Self {
			hwnd_wnd_base: null!(),
			wb_items: Vec::<Box<dyn WbItem>>::new(),
//			wb_items: Vec::<Rc<dyn WbItem>>::new(),
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
					if w_param as i32 != WndBase::ID_BUTTON {
						return 0;
					}
										
					let box_wb_item: &mut Box<dyn WbItem>
						= &mut *(GetWindowLongPtrW(l_param as HWND, GWLP_USERDATA) as *mut Box<dyn WbItem>);
					box_wb_item.on_click();
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
	pub fn yield_WbButton<T: 'static>(&mut self, x: i32, y: i32, caption: &str) -> &mut WbButton<T> {
		let wb_button = Box::new(WbButton::<T>::new(x, y, caption));
		self.wb_items.push(wb_button);
		self.wb_items.last_mut().unwrap().leak_from_box::<WbButton<T>>()
	}
/*
	#[allow(non_snake_case)]
	pub fn yield_WbButton<T: 'static>(&mut self, x: i32, y: i32, caption: &str) -> Rc<WbButton<T>> {
		let wb_button: Rc<WbButton<T>> = Rc::new(WbButton::<T>::new(x, y, caption));
		let ret_val: Rc<WbButton<T>> = Rc::clone(&wb_button);
		self.wb_items.push(wb_button);
		ret_val
	}
*/

	#[allow(non_snake_case)]
	pub fn yield_WbTextBox(&mut self, x: i32, y: i32, width: i32, height: i32) -> &mut WbTextBox {
		let wb_text_box = Box::new(WbTextBox::new(x, y, width, height));
		self.wb_items.push(wb_text_box);
		self.wb_items.last_mut().unwrap().leak_from_box::<WbTextBox>()
	}
}

////////////////////////////////////////////////////////////////
// WbButton
pub struct WbButton<T> {
	wb_item_base: WbItemBase,
	
	caption: String,
	tgt_obj: Option<Rc<RefCell<T>>>,
	tgt_fn: Option<fn(&mut T)>,
}

impl<T> WbButton<T> {
	// 将来、x, y はグリッド指定に変更する
	pub fn new(x: i32, y: i32, caption: &str) -> Self {
		Self {
			wb_item_base: WbItemBase::with_xy_size(x, y, 100, 25),
			
			caption: String::from(caption),
			tgt_obj: None,
			tgt_fn: None,
		}
	}
	
	pub fn set_handler(&mut self, tgt_obj: &Rc<RefCell<T>>, tgt_fn: fn(&mut T)) -> &mut Self {	
		if self.tgt_obj.is_some() {
			panic!("\n!!! ２重登録 -> self.tgt_obj\n");
		}	
		if self.tgt_fn.is_some() {
			panic!("\n!!! ２重登録 -> self.tgt_fn\n");
		}
		
		self.tgt_obj = Some(Rc::clone(tgt_obj));
		self.tgt_fn = Some(tgt_fn);
		self
	}
}

impl<T> WbItem for WbButton<T> {
	fn wb_item_base(&mut self) -> &mut WbItemBase {
		&mut self.wb_item_base
	}
	
	fn on_crt_parent_wnd_rsc(&mut self, ptr_boxed_wb_item: isize, hwnd_parent: HWND) {
		let caption = str_to_u16(&self.caption);
		
		let wb_item_base = self.wb_item_base(); 
		unsafe {
			let hwnd = CreateWindowExW(
				0,  // ex_style
				u!("BUTTON").as_ptr(),  // window class name
				caption.as_ptr(),  // title (caption)
				WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
				wb_item_base.x.unwrap(), wb_item_base.y.unwrap(),  // x, y
				wb_item_base.width.unwrap(), wb_item_base.height.unwrap(), // width, height
				hwnd_parent,  // hWndParent
				WndBase::ID_BUTTON as HMENU,
				null!(),  // hInstance
				null!()   // lpParam
			);
			
			if hwnd == null!() {
				panic!("!!! hwnd == null!()");
			}
			
//			let x = (self as *const WbButton<T>) as *const dyn WbItem;
			
			SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr_boxed_wb_item);
		}
	}

	fn on_click(&mut self) {
		let mut tgt_obj: RefMut<'_, T> = self.tgt_obj.as_ref().unwrap().borrow_mut();
		(self.tgt_fn.unwrap())(tgt_obj.deref_mut());
	}
}

pub trait WbButtonBase<T> {
	fn set_handler(&mut self, tgt_obj: &Rc<RefCell<T>>, tgt_fn: fn(&mut T)) -> &mut Self;
}


////////////////////////////////////////////////////////////////
// WbTextBox
pub struct WbTextBox {
	wb_item_base: WbItemBase,
}

impl WbTextBox{
	// 将来、x, y はグリッド指定に変更する
	pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
		Self {
			wb_item_base: WbItemBase::with_xy_size(x, y, width, height),
		}
	}
}

impl WbItem for WbTextBox {
	fn wb_item_base(&mut self) -> &mut WbItemBase {
		&mut self.wb_item_base
	}

	fn on_crt_parent_wnd_rsc(&mut self, ptr_box_wb_item: isize, hwnd_parent: HWND) {
		let wb_item_base = self.wb_item_base(); 
		unsafe {
			let hwnd = CreateWindowExW(
				0,  // ex_style
				u!("EDIT").as_ptr(),  // window class name
				u!("").as_ptr(),  // title (caption)
				WS_CHILD | WS_VISIBLE | WS_BORDER | ES_LEFT as u32 | ES_MULTILINE as u32 | WS_HSCROLL | WS_VSCROLL as u32,
				wb_item_base.x.unwrap(), wb_item_base.y.unwrap(),  // x, y
				wb_item_base.width.unwrap(), wb_item_base.height.unwrap(), // width, height
				hwnd_parent,  // hWndParent
				WndBase::ID_TBOX as HMENU,
				null!(),  // hInstance
				null!()   // lpParam
			);
			
			if hwnd == null!() {
				panic!("!!! hwnd == null!()");
			}
			
			SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr_box_wb_item);
		}
	}
}

