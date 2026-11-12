int APIENTRY wWinMain(HINSTANCE hInst, HINSTANCE hPrevInst, LPWSTR lpCmdLine, int nCmdShow)
{
	try
	{
		KOL_Wnd_Factory<MainWnd> factory_MainWnd{};
    
		// New_OL_Wnd_Wp() は、メモリ内にオブジェクトを生成するのみで、ウィンドウリソース等は生成しない
		// Crt_WndRsc() で、ウィンドウリソースが生成される
		factory_KOL_MainWnd.New_OL_Wnd_Wp
      ->Crt_WndRsc(KOL_Wnd_Pos{ 950, 2180 })
      ->Show_Wnd();
    
		// ----------------------------------------------------
		for (MSG msg; GetMessage(&msg, NULL, 0, 0); )
		{
			try
			{
				if (msg.message == WM_APP)
				{
					GUIcue_Handler::Execute_on_GUI_thread((GUIcue_Msg*)msg.lParam);
					continue;
				}

				TranslateMessage(&msg);
				DispatchMessage(&msg);
			}
			catch (const KException& ex)
			{
				g_StdOut <= ex;
			}
		}
	}
	catch (const KException& ex)
	{
		OutputDebugString(L"\r\n!!! KException を GetMessage() ループの外側でキャッチしました。\r\n\r\n");
		ex.DbgOut();
	}
	return 0;
}
