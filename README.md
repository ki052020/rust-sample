# rust-sample（Windows API 用となっています）
以下の画像にあるような、為替や株式などのチャート分析ツールを作ることが最終目標です。<br>
Rust 初心者、Windows API 初心者を対象としています。私の備忘録ともなっています^^。

![screen](https://github.com/user-attachments/assets/ec461881-370c-413b-8cb8-3f70c884c212)


## Windows 専用となった背景
最初は GTK を利用して開発を始めました。
GTK はよく整備されているライブラリであるため、大変使いやすいです。しかし、以下の２つのことが問題に感じるようになりました。

1. コンパイルが遅い
- テストコードを書くときなど、数行書いてビルド、実行、ということを繰り返すのが私の開発スタイルです。
しかし、GTK のライブラリが大きいため、数行書いてビルドをするたびに 10秒、20秒と待たされることが多くあり、これが苦痛に感じました。

2. 描画速度が遅い
- 4K ディスプレイでの表示を行っていることもあって、最低限の情報を表示するだけで、描画命令を２万回以上コールします。<br>
私の利用目的は「分析値変更、即時チャート表示、チェックしたいところまで即スクロールして確認」というものです。<br>
しかし、GTK はマルチプラットフォーム用であるためだろうと思いますが、表示、スクロールがもっさりとなってしまい、分析入力値を少しずつ変更して再表示させることにためらいを感じました。

以上のことにより、Windows API を直接叩くようにしたので、Windows 専用となってしまいました。

## Windows への Rust のインストール
Mac、Linux ならば、コマンド１発で Rust がインストールできます。羨ましい。<br>
Windows の場合、原則としてインストールは３段階に分かれます。

1. **Visual Studio をインストール**
- C コンパイラ等をインストールするためです。
2. **Rust コンパイラをインストール**

3. **Visual Studio Code をインストール**（省略可）
- 上記の Visual Studio と名前が似ていてややこしいですが、Visual Studio Code はコード作成支援機能を持つテキストエディタです。<br>
自分の好きなテキストエディタがあれば、これはインストールしなくても大丈夫です。

上記の手順は、以下のMicorsoft 公式のページを参考にしてインストールするのが一番簡単だと思います。<br>
https://learn.microsoft.com/ja-jp/windows/dev-environment/rust/setup

## Cargo を使ってみよう

TODO：cargo の詳細は後で書く

```
> cargo init rust-sample
> cd rust-sample
> cargo run
```

`Hello, world!` と表示されたらＯＫ！。

## MessageBox を使ってみよう
Windows API を利用して、`Hello, world!` をウィンドウで表示してみましょう。<br>
`main.rs` を書きかえてみます。

```Rust
fn main() {
	let title = "TEST\0";
	let message = "Hello, World!\0";
	unsafe {
		MessageBoxA(std::ptr::null_mut(), message, title, MB_OK);
	}
}

```

Windows API の呼び出しは `unsafe` になるので、とりあえず `MessageBoxA()` は `unsafe` で囲ってあげます。<br>
`MessageBoxA()` については、以下の Microsoft 公式の URL が参考になります。Windows API を調べたい場合は、左上にある検索ボックスに API を入れると検索できるので便利^^。<br>
https://learn.microsoft.com/ja-jp/windows/win32/api/winuser/nf-winuser-messageboxa

上記の URL を見ると、`MessageBoxA()` の最後の `A` とか、`HWND` とかなんだ、となりますが、それは後々調べれば良いです。

さて、エディタ上では赤線が表示されていてエラーがあることが分かりますが、とりあえず build してみましょう。

```
> cargo build
```

いろいろエラーが出ると思います。では、例えば、`MB_OK` のところに `not found in this scope` となっているので、これを解消します。<br>
以下の URL の上にある検索ボックスに「MB_OK」と入力すると、いくつか候補が表示されるので、とりあえず一番上をクリックしてみます。<br>
https://docs.rs/windows-sys/latest/windows_sys/index.html

すると、上の方に `windows_sys::Win32::UI::WindowsAndMessaging` というものが表示されるので、これをソースコードに組み込めば良いのです。（<- 雑な説明ですすみません）

`main.rs` に、以下のように先頭に１行追加します。

```
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn main() {
	let title = "TEST\0";
	let message = "Hello, World!\0";
	unsafe {
		MessageBoxA(std::ptr::null_mut(), message, title, MB_OK);
	}
}
```

この状態で再び `cargo build` とすると、今度は `use windows_sys::...` の部分に、`use of undeclared crate ...` と表示されます。<br>
crate というのは、Rust における「開発単位」になります。例えば、今私達が作っているのは、`rust-sample crate` となります。

私達が作っている crate の中にあるものならば、`rust-sample:: ...` となるのですが、私たちが作っているものの中にない `windows-sys:: ...` という `windows-sys crate` を利用しようとしているのでエラーになるのです。<br>
そのような自分が開発している crate にないものを利用する場合には、`Cargo.toml` に、それを使うことを宣言しなくてはなりません。

`Cargo.toml` を以下のように書き換えましょう。

```
[package]
name = "rust-sample"
version = "0.1.0"
edition = "2024"

[dependencies.windows-sys]
version = "*"
features = [
	"Win32_UI_WindowsAndMessaging",
]
```

`windows_sys::Win32::UI::WindowsAndMessaging` を追加したい場合は、`[dependencies.windows-sys]` というセクションを作って、その中に `features` として、`::` を `_` に置き換えたものを書いておけばＯＫです。

この状態で `cargo build` します。<br>
まだエラーは出ますが、`not found` というようなエラーは出なくなったと思います。<br>
Rust コンパイラに、Windows API を認識させることができるようになったことは、一歩前進です！

さて、`cargo build` のエラーを見てみると、`MessageBoxA(std::ptr::null_mut(), message, title, MB_OK);` の `message` のところに `expected raw pointer *const u8` と表示されています。<br>
`message` の部分には `*const u8` が期待されているということです。

Rust に慣れていないと、`*const u8` と `&str` の違いが分かりにくいですが、とりあえず、`*const u8` に変換したい場合、`as_ptr()` が利用できると覚えておけば良いです。

ということで、`MessageBoxA` の部分を以下のように書き換えてみましょう。

```
MessageBoxA(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK);
```

上記のように書き換えて、`cargo build` してみましょう。うまくいくはずです。<br>
この状態で `cargo run` として、メッセージウィンドウを表示してみましょう！

```
> cargo run
```

続き -> [MessageBoxW](https://github.com/ki052020/rust-sample/blob/main/010_MessageBoxW.md)

