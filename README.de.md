[简体中文](README.md) | [English](README.en.md) | [繁體中文](README.zh-TW.md) | **Deutsch** | [Français](README.fr.md) | [日本語](README.ja.md) | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **KI-Terminal-Manager mit gestapelten Multi-Fenstern** — macht jedes AI-Coding- oder Agent-Tool zu einer multi-instanzfähigen, fernsteuerbaren Smart-Workstation.

![OpenNex Übersicht](docs/screenshots/panorama.png)

| | |
|---|---|
| **1000+** | gleichzeitig aktive Fenster |
| **20+** | Oberflächensprachen |
| **3** | native Plattformen (Linux / Windows / macOS) |

## Screenshots

**🤖 KI-Assistent · Modellkonfiguration**

![](docs/screenshots/ai-config.png)

Modell-API und Kontextlimit konfigurieren, um den KI-Assistenten zu aktivieren.

**🤖 KI-Assistent · Terminalgsteuerte Konversation**

![](docs/screenshots/ai-chat-terminal.png)

Nach der API-Konfiguration genügt ein Dialog, damit die KI Befehle im Terminal ausführt oder Probleme untersucht — jederzeit eingreifbar.

**🌐 Fernsteuerungs-Assistent**

![](docs/screenshots/remote-control.png)

Per QR-Code oder Browserverbindung das Terminal fernbedienen; auch im Terminal gestartete Programme sind fernsteuerbar.

**🔒 Arbeitsbereich-Sperre**

![](docs/screenshots/workspace-lock.png)

Arbeitsbereich mit einem Klick sperren, schnell wieder entsperren.

**★ Befehlsfavoriten**

![](docs/screenshots/command-favorites.png)

Häufig genutzte Befehle in Ordnern sammeln, griffbereit zum Aufruf.

**↩ Befehls-Autovervollständigung**

![](docs/screenshots/auto-complete.png)

Passt sich beim Tippen an: Systembefehle plus der Verlauf dieses Terminals.

**◉ Aktivitätsanzeige**

![](docs/screenshots/idle-busy-indicator.png)

Rot = mindestens ein Terminal des Arbeitsbereichs ist aktiv; Grün = alle fertig.

## Funktionen

**✦ KI-Ausführung**
Die KI plant und führt Terminalaufgaben selbstständig aus — jederzeit unterbrech- und pausierbar.

**🌐 Fernsteuerung über WAN**
Terminal-Sitzungen von Handy, Tablet oder PC außerhalb des LANs übernehmen.

**▦ Flexibles Fensterlayout**
Fenster frei anordnen und stapeln; Layouts speichern & laden.

**↩ Befehlsverlauf**
Unbegrenzter Verlauf mit Ein-Klick-Aufruf — kein erneutes Tippen.

**★ Favoriten**
Häufig genutzte Befehle mit einem Klick griffbereit.

**◈ Designs**
Eigene Themes für den persönlichen Terminal-Look.

**🔒 Arbeitsbereich-Verschlüsselung**
Master-Passwort-Sperre und verschlüsselte Arbeitsbereiche.

**⇄ Fernsteuerung im LAN**
Terminal-Sitzungen von jedem Gerät im selben Netzwerk übernehmen.

**◉ Aktivitätserkennung**
Live-Status (aktiv/ausgelastet) je Arbeitsbereich.

**⇅ SSH-Hosts**
Integriertes SSH-Host-Management für direkten Zugriff.

**⚡ Performance-Monitoring**
Live-Ressourcenverbrauch von Fenstern und System.

**⟲ Arbeitsbereich-Gedächtnis**
Zustand wird automatisch gesichert und beim Neustart wiederhergestellt.

**🖥 Systeminfo**
Systeminformationen und Ressourcen in Echtzeit.

**⤺ Pfad-Wiederherstellung**
Arbeitsverzeichnisse werden nach dem Neustart automatisch wiederhergestellt.

**🌐 Sprachen**
20+ Oberflächensprachen mit Ein-Klick-Wechsel.

## Plattformunterstützung

| Plattform | Minimum | Installation |
|------|---------|---------|
| Windows | Windows 10+ | NSIS-Installer / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

Alle Installationspakete gibt es auf der [Website](https://opennex.zeadix.com) oder unter [GitHub Releases](https://github.com/zeadix/opennex/releases/latest); die App enthält signierte Ein-Klick-Updates.

## Entwicklung & Testing

```bash
cd tauri-app

# Frontend-Abhängigkeiten installieren (erstmalig)
npm install

# Dev-Modus: vite Hot Reload + Rust-Debug-Build (oder ./dev.sh)
npm run tauri dev

# Frontend-Typecheck / Produktions-Build
npx tsc --noEmit
npm run build

# Paket erstellen (aktuelle Plattform)
npm run tauri build
```

### CI/CD

Ein Push eines `v*.*.*`-Tags löst GitHub Actions aus: Multi-Plattform-Builds (NSIS+MSI / DMG / DEB+AppImage), minisign-signierte Update-Pakete, Updater-Feed `tauri/latest.json` auf R2, ein GitHub Release und ein Website-Rebuild:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Projektstruktur

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