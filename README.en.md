[简体中文](README.md) | **English** | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [日本語](README.ja.md) | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **multi-window stacked AI terminal manager** — turning any AI Coding or Agent tool into a multi-instance, remotely controllable smart workstation.

![OpenNex panorama](docs/screenshots/panorama.png)

| | |
|---|---|
| **1000+** | concurrent active windows |
| **20+** | UI languages |
| **3** | native platforms (Linux / Windows / macOS) |

## Screenshots

**🤖 AI assistant · Model configuration**

![](docs/screenshots/ai-config.png)

Configure the model API and context limits to enable the AI assistant.

**🤖 AI assistant · Conversational terminal control**

![](docs/screenshots/ai-chat-terminal.png)

Once the model API is configured, just ask the AI to run commands or debug issues in your terminal — you can intervene at any time.

**🌐 Remote control helper**

![](docs/screenshots/remote-control.png)

Scan the QR code or connect from a browser to operate your terminal remotely; apps launched inside the terminal can be remote-controlled too.

**🔒 Workspace lock**

![](docs/screenshots/workspace-lock.png)

Lock your workspace with one click while you're away; unlock quickly when you're back.

**★ Command favorites**

![](docs/screenshots/command-favorites.png)

Collect frequently used commands into folders for quick access.

**↩ Command auto-completion**

![](docs/screenshots/auto-complete.png)

Matches as you type: system commands plus this terminal's own history.

**◉ Idle/busy indicator**

![](docs/screenshots/idle-busy-indicator.png)

Red = at least one terminal in the workspace is still busy; green = all terminals are idle and jobs have finished.

## Key Features

**✦ AI auto-execution**
The AI plans and runs terminal tasks on its own — interruptible and pausable throughout.

**🌐 WAN remote control**
Take over terminal sessions from phones, tablets and PCs beyond the LAN.

**▦ Flexible window layout**
Freely arrange and stack terminal windows; layouts save & load, keeping multi-instance AI tools tidy.

**↩ Command history**
Unlimited command history with one-keystroke recall — no more retyping.

**★ Favorites**
One-click favorites for your most-used commands.

**◈ Themes**
Custom themes for a personal terminal look.

**🔒 Workspace encryption**
Master-password lock screen and encrypted workspaces for sensitive environments.

**⇄ LAN remote control**
Control terminal sessions from any device on the same LAN.

**◉ Idle/busy detection**
Live idle/busy status for every workspace at a glance.

**⇅ SSH hosts**
Built-in SSH host management for one-click access to remote servers.

**⚡ Performance monitoring**
Live window and system resource monitoring.

**⟲ Workspace memory**
Workspace state auto-saves and restores on relaunch.

**🖥 System info**
Real-time host system info and resource usage.

**⤺ Path restore**
Working directories restore automatically after restart.

**🌐 Languages**
20+ UI languages with one-click switching.

## Platform Support

| Platform | Minimum | Install |
|------|---------|---------|
| Windows | Windows 10+ | NSIS installer / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

Download from the [website](https://opennex.zeadix.com) or [GitHub Releases](https://github.com/zeadix/opennex/releases/latest); the app ships with signature-verified one-click updates.

## Development & Testing

```bash
cd tauri-app

# install frontend deps (first run)
npm install

# dev mode: vite hot reload + Rust debug build (or just ./dev.sh)
npm run tauri dev

# frontend typecheck / production build
npx tsc --noEmit
npm run build

# package (current platform)
npm run tauri build
```

### CI/CD

Pushing a `v*.*.*` tag triggers GitHub Actions: multi-platform packaging (NSIS+MSI / DMG / DEB+AppImage), minisign-signed update packages, the updater feed `tauri/latest.json` uploaded to R2, a GitHub Release, and a website rebuild:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Project Structure

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