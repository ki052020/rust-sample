export class MainWnd
	: public KOL_StdWnd
{
public:
	MainWnd();
	virtual ~MainWnd() {}

	void Crt_RightSide_PNL(KW_Panel_Dn* wpPnlDn_parent);

  // ---------------------------------------------------------------------
	void OnBtn_Test();
	virtual void On_WM_DESTROY() override { PostQuitMessage(0); }
};

////////////////////////////////////////////////////////////////////////////////////////
module : private;

MainWnd::MainWnd()
	: KOL_StdWnd{ NEW KW_Panel_Rt{ NULL }, L"main window" }
{
	//「StdOut」用の KW_TextBox
	m_apTop_panel->NewItem_Wp<KW_TextBox>()
		->Swallow_pFnObj_OnCrted_WndRsc(
				NEW FnObj_OnCrted_WndRsc{ [](KW_TextBox* pKW_TextBox) { g_StdOut.Set_hWnd(pKW_TextBox->Get_hWnd()); } })
		->CltSize(600, 1000);

	// 右側パネルの生成
	KW_Panel_Dn* wpPnlDn_RightSide = m_apTop_panel->NewItem_Wp<KW_Panel_Dn>();
	this->Crt_RightSide_PNL(wpPnlDn_RightSide);
}

// -------------------------------------------------------------------------------------
void MainWnd::Crt_RightSide_PNL(KW_Panel_Dn* const wpPnlDn_parent)
{
	KW_Panel* wpPnl_top_Btns = wpPnlDn_parent->NewItem_Wp<KW_Panel_Rt>();
	wpPnl_top_Btns->m_Mgn.m_Top = 5;

	//「クリア」ボタン
	wpPnl_top_Btns->NewItem_Wp<KW_Button>(L"Dispクリア", NEW FnObj_OnClkBtn{ []{ g_StdOut.Clear(); } })
		->CltWidth(80);

	//「テスト」ボタン
	wpPnl_top_Btns->NewItem_Wp<KW_Button>(L"テスト", NEW FnObj_OnClkBtn{ [this]{ this->OnBtn_Test(); } })
		->CltWidth(80);

	//「Close」ボタン
	wpPnl_top_Btns->NewItem_Wp<KW_Button>(L"Close", NEW FnObj_OnClkBtn{ [this]{ this->Destroy_Wnd(); } })
		->CltWidth(80)
		->Add_AlignFlag(EN_Extend_Width | EN_Align_Right)
		->Mgn_Right(10);
}

////////////////////////////////////////////////////////////////////////////////////////
// OnBtn_Test()
void MainWnd::OnBtn_Test()
{
	g_StdOut <= L"\r\n+++ 現在このボタンには機能は割り当てられていません。\r\n" <= FILE_LINE_RR;
}
