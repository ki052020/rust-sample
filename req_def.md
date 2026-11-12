060_TextBox_v0.1.md を拡張したい。
パネルクラスを追加して、C++ で書くと、以下のようなコードが実現できるようにしたい。
以下のコードで、wpPnl_top_Btns は、パネルクラスを表すものとする。

参考として、「C++」ディレクトリを参照すること。

//「Close」ボタン

wpPnl_top_Btns->NewItem_Wp<KW_Button>(L"Close", NEW FnObj_OnClkBtn{ [this]{ this->Destroy_Wnd(); } })
	->CltWidth(80);
