```Rust
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
			cbSize: std::mem::size_of::<ACTCTXW> as u32,
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
//			dwICC: ICC_STANDARD_CLASSES,
			dwICC: 0,
		};
		if InitCommonControlsEx(&icc) == FALSE {
			panic!("\n!!! failed -> InitCommonControlsEx()\n");
		}
*/
	}
}
```
