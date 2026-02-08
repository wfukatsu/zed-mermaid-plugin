# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Zed拡張機能で、Markdownファイル内のMermaidダイアグラムをインラインSVGとしてレンダリングします。
参考実装: https://github.com/dawsh2/zed-mermaid-preview

## Architecture

### 2層アーキテクチャ

1. **Zed Extension Layer** (`src/lib.rs`)
   - `zed_extension_api` を使用したZedエディタとの統合
   - LSPバイナリのダウンロード・管理・バージョン管理
   - Mermaid CLI (`mmdc`) の自動インストール
   - 開発用と本番用のバイナリ解決戦略

2. **LSP Server Layer** (`lsp/src/`)
   - `main.rs`: LSPプロトコルハンドラ、コマンド処理、ドキュメント管理
   - `render.rs`: Mermaidコードのレンダリングロジック、セキュリティサニタイゼーション
   - `mermaid-config.json`: Mermaid実行時設定

### データフロー

```
Zedエディタ
  ↓ (CodeActionリクエスト)
LSP Server
  ↓ (コードブロック検出)
render.rs
  ↓ (一時ファイル生成)
mmdc CLI (Node.js)
  ↓ (SVG生成)
セキュリティサニタイゼーション
  ↓ (HTMLコメント付きでSVG挿入)
workspace/applyEdit
  ↓
Zedエディタに反映
```

### ファイル構成

- `.mermaid/` - レンダリング結果のSVGと編集用.mmdファイルを保存
- `mermaid-lsp-cache/[version]/` - ダウンロードしたLSPバイナリのキャッシュ

## Development Commands

### ビルド
```bash
# LSPサーバーをビルド
cd lsp && cargo build --release

# 拡張機能をビルド（cdylib）
cargo build --release

# 推奨: スクリプトを使用
./scripts/build.sh
```

### テスト
```bash
# LSPサーバーのユニットテスト
cd lsp && cargo test

# 統合テスト
cargo test
```

### インストール
```bash
# ローカル開発用インストール
./scripts/install.sh

# または手動でZedに登録
# Zed: Cmd+Shift+P → "Extensions: Install Development Extension"
```

### 開発セットアップ
```bash
# 初回セットアップ（依存関係インストール）
./scripts/dev-setup.sh

# 環境変数を設定して開発版を使用
export MERMAID_LSP_PATH=/path/to/your/mermaid-lsp
```

## Key Components

### Extension (`src/lib.rs`)

**`MermaidPreviewExtension` 構造体**
- `new()`: 初期化時にLSPバイナリを事前ダウンロード（遅延回避）
- `language_server_command()`: LSPサーバープロセスの起動コマンドを返す
- `get_lsp_path()`: バイナリ解決（優先順位: 環境変数 > PATH > キャッシュ > ダウンロード）
- `ensure_mermaid_cli()`: Mermaid CLI (mmdc) の自動インストール

**バイナリ管理**
- `download_lsp_binary()`: GitHub Releasesから最新版を自動ダウンロード
- `match_asset()`: プラットフォーム別アセット選択（Mac/Linux/Windows, x86_64/aarch64）
- `purge_old_cache_versions()`: 古いバージョンの自動削除

### LSP Server (`lsp/src/main.rs`)

**リクエスト処理**
- `textDocument/codeAction`: Mermaidコードブロックにレンダリングアクションを提供
- `workspace/executeCommand`: 4つのコマンド実装
  - `mermaid.renderAllLightweight`: 全ダイアグラムをレンダリング
  - `mermaid.renderSingle`: 単一ダイアグラムをレンダリング
  - `mermaid.editAllSources`: 全ソースを編集モードに復元
  - `mermaid.editSingleSource`: 単一ソースを編集モードに復元

**通知処理**
- `textDocument/didOpen/didChange/didClose`: メモリ内ドキュメント状態管理

**コードブロック検出**
- `find_mermaid_fence()`: Markdown中の```mermaidブロックを正規表現で検出
- キャッシング: コードハッシュでレンダリング結果をキャッシュ（不要な再実行を回避）

### Rendering (`lsp/src/render.rs`)

**レンダリングプロセス**
1. 一時ディレクトリ作成（`tempfile::tempdir()`）
2. `diagram.mmd`と`mermaid-config.json`を生成
3. `mmdc`コマンド実行（引数ベース、シェルインジェクション対策）
4. SVG出力のセキュリティサニタイゼーション

**セキュリティ対策**
- `<script>`タグ検出・拒否（大文字小文字非依存）
- イベントハンドラ除去（onclick, onmouseover等）
- `javascript:`プロトコル除去
- `<foreignObject>`をネイティブSVGテキストに変換（埋め込みHTML実行防止）
- パストラバーサル攻撃対策
- 正規表現のカタストロフィックバックトラック対策（プリコンパイル、非貪欲マッチ）

**設定**
- `htmlLabels: false` - HTMLラベルによるXSS回避

## Technology Stack

### Rust Dependencies

**拡張機能（`Cargo.toml`）:**
- `zed_extension_api` (0.1.0) - Zed拡張API
- `dirs` (5) - クロスプラットフォームディレクトリパス

**LSPサーバー（`lsp/Cargo.toml`）:**
- `lsp-server` (0.7) - LSPプロトコル実装
- `lsp-types` (0.95) - LSP型定義
- `tokio` (1.0) - 非同期ランタイム
- `serde`/`serde_json` - JSONシリアライゼーション
- `anyhow` - エラーハンドリング
- `regex` (1.10) - 正規表現
- `which` (5) - 実行ファイル検索
- `tempfile` (3.10) - 一時ファイル管理
- `html-escape` (0.2) - HTMLエスケープ
- `base64` (0.22) - エンコーディング
- `url` (2.0) - URL処理
- `chrono` (0.4) - 日時処理
- `log`/`env_logger` - ロギング

### External Tools
- **Mermaid CLI (`mmdc`)**: Node.js製のダイアグラムレンダリングツール（npm経由で自動インストール）

## Extension Configuration

**`extension.toml`:**
```toml
id = "mermaid-preview"
name = "Mermaid Preview"
version = "0.1.24"
schema_version = 1

[compatibility]
zed = "0.210.0"  # 最小要求バージョン

[language_servers.mermaid]
name = "Mermaid LSP"
languages = ["Markdown", "Mermaid"]
```

## Development Workflow

### ローカル開発
1. `MERMAID_LSP_PATH`環境変数を設定
2. `./scripts/build.sh`でビルド
3. `./scripts/install.sh`でZedに登録
4. Zedを再起動して拡張機能をリロード

### 本番リリース
1. GitHub Releasesにプラットフォーム別バイナリをアップロード
2. 拡張機能が自動的に最新版をダウンロード
3. `mermaid-lsp-cache/[version]/`にキャッシュ

### デバッグ
- LSPサーバーログ: 環境変数 `RUST_LOG=debug` で詳細ログ出力
- ファイルトレース: `.mermaid/`ディレクトリ内の生成ファイルを確認

## Security Considerations

- **信頼できるソースからのコードのみ**: Mermaidコードは信頼できる入力を前提
- **SVGサニタイゼーション**: スクリプトタグ、イベントハンドラ、JavaScriptプロトコルを除去
- **パストラバーサル対策**: ファイルパス検証を実装
- **コマンドインジェクション対策**: 引数ベースでmmdcを実行、シェルを介さない
- **リソース制限**: 複雑なダイアグラムによる枯渇には未対応（mmdc自体の制限に依存）

## Known Limitations

- mmdc バイナリ自体の脆弱性には対応不可
- 非常に複雑なダイアグラムでのリソース消費
- エラーハンドリングは基本的（mmdc失敗時のフォールバック無し）
