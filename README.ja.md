[简体中文](README.md) | [English](README.en.md) | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | **日本語** | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **マルチウィンドウ積層式 AI ターミナルマネージャー** — あらゆる AI コーディング／エージェントツールを、複数起動・遠隔操作可能なスマートなワークステーションに。

| | |
|---|---|
| **1000+** | 同時アクティブウィンドウ |
| **20+** | UI 言語 |
| **3** | ネイティブ対応プラットフォーム（Linux / Windows / macOS） |

## 主な機能

**✦ AI 自動実行**
AI が终端タスクを自律的に計画・実行。いつでも介入・一時停止できます。

**🌐 WAN リモート操作**
LAN を超えて、スマホ・タブレット・PC からターミナルセッションを遠隔操作。

**▦ 柔軟なウィンドウレイアウト**
ウィンドウを自由に配置・積層。レイアウトは保存・復元でき、AI ツールの多重起動も整然。

**↩ コマンド履歴**
入力したコマンドを無限に記録。キー一つで呼び出し、再入力は不要。

**★ お気に入り**
よく使うコマンドをワンクリックで呼び出し。

**◈ テーマ**
カスタムテーマでターミナルを自分好みに。

**🔒 ワークスペース暗号化**
マスターパスワードロックと暗号化で機密環境も安心。

**⇄ LAN リモート操作**
同一 LAN 内のスマホ・タブレット・PC からセッションを遠隔操作。

**◉ 繁閒検知**
ワークスペースごとの稼働状態をリアルタイム表示。

**⇅ SSH ホスト**
SSH ホスト管理を内蔵。リモートサーバーにワンクリック接続。

**⚡ パフォーマンス監視**
ウィンドウとシステムリソースをリアルタイム監視。

**⟲ ワークスペース記憶**
状態を自動保存。再起動後も前回の現場を復元。

**🖥 システム情報**
ホストのシステム情報とリソースをリアルタイム表示。

**⤺ パス復元**
再起動後も各ウィンドウの作業ディレクトリを自動復元。

**🌐 多言語**
20+ の UI 言語をワンクリックで切替。

## 対応プラットフォーム

| プラットフォーム | 最小要件 | インストール |
|------|---------|---------|
| Windows | Windows 10+ | NSIS インストーラー / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

すべてのインストーラーは [公式サイト](https://opennex.zeadix.com) または [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) から。アプリ内蔵の署名検証付きワンクリック更新に対応。

## 開発とテスト

```bash
cd tauri-app

# フロントエンド依存のインストール（初回）
npm install

# 開発モード：vite ホットリロード + Rust デバッグビルド（./dev.sh でも可）
npm run tauri dev

# フロントエンド型チェック / 本番ビルド
npx tsc --noEmit
npm run build

# パッケージ化（現在のプラットフォーム）
npm run tauri build
```

### CI/CD

`v*.*.*` 形式のタグを push すると GitHub Actions が起動：マルチプラットフォーム パッケージ化（NSIS+MSI / DMG / DEB+AppImage）、minisign 署名付き更新パッケージ、更新フィード `tauri/latest.json` の R2 アップロード、GitHub Release 発行、サイト再ビルド：

```bash
git tag v0.2.0
git push origin v0.2.0
```

## プロジェクト構成

```
opennex/
├── .github/workflows/
│   ├── release.yml                # release: multi-platform build + signing + R2 + updater feed + GitHub Release
│   └── rebuild-updater-feed.yml   # manual tool: rebuild the updater feed from R2 (fix without a release)
├── assets/icon/                   # app icon source
├── tauri-app/                     # Tauri client (the actively developed line)
│   ├── src/                       # React + TypeScript frontend (xterm.js terminal)
│   ├── src-tauri/                 # Tauri 2 Rust backend (PTY / history / AI proxy / updater)
│   └── dev.sh                     # one-shot dev launcher (vite + cargo run)
└── research/                      # research & audit notes
```

## License

MIT
