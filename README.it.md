[简体中文](README.md) | [English](README.en.md) | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [日本語](README.ja.md) | **Italiano** | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **gestore di terminali IA con finestre impilate** — trasforma qualsiasi strumento di coding IA o agente in una workstation multi-istanza controllabile da remoto.

| | |
|---|---|
| **1000+** | finestre attive simultanee |
| **20+** | lingue dell'interfaccia |
| **3** | piattaforme native (Linux / Windows / macOS) |

## Funzionalità principali

**✦ Esecuzione IA**
L'IA pianifica ed esegue le attività del terminale — interrompibile e pausabile in ogni momento.

**🌐 Controllo remoto WAN**
Gestisci sessioni da telefono, tablet e PC oltre la LAN.

**▦ Layout flessibile**
Disponi e impila le finestre liberamente; i layout si salvano e si ricaricano.

**↩ Cronologia comandi**
Cronologia illimitata con richiamo a un tasto.

**★ Preferiti**
I comandi più usati a un clic.

**◈ Temi**
Temi personalizzati per un terminale a tua immagine.

**🔒 Crittografia degli spazi**
Blocco con password principale e spazi di lavoro cifrati.

**⇄ Controllo remoto LAN**
Gestisci le sessioni da qualsiasi dispositivo sulla stessa rete.

**◉ Rilevamento attività**
Stato live (attivo/occupato) di ogni spazio di lavoro.

**⇅ Host SSH**
Gestione host SSH integrata, accesso con un clic.

**⚡ Monitoraggio prestazioni**
Risorse di finestre e sistema in tempo reale.

**⟲ Memoria degli spazi**
Lo stato viene salvato e ripristinato al riavvio.

**🖥 Info di sistema**
Informazioni e risorse della macchina in tempo reale.

**⤺ Ripristino percorsi**
Le directory di lavoro si ripristinano automaticamente dopo il riavvio.

**🌐 Multilingua**
Oltre 20 lingue dell'interfaccia con un clic.

## Piattaforme supportate

| Piattaforma | Minimo | Installazione |
|------|---------|---------|
| Windows | Windows 10+ | Installer NSIS / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

Scarica dal [sito](https://opennex.zeadix.com) o da [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) ; aggiornamenti integrati verificati tramite firma.

## Sviluppo e test

```bash
cd tauri-app

# installa le dipendenze front (prima volta)
npm install

# modalità dev: hot reload vite + build Rust debug (oppure ./dev.sh)
npm run tauri dev

# typecheck frontend / build di produzione
npx tsc --noEmit
npm run build

# pacchetto (piattaforma corrente)
npm run tauri build
```

### CI/CD

Il push di un tag `v*.*.*` avvia GitHub Actions: build multipiattaforma (NSIS+MSI / DMG / DEB+AppImage), pacchetti di aggiornamento firmati minisign, feed di aggiornamento `tauri/latest.json` su R2, GitHub Release e ricostruzione del sito:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Struttura del progetto

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
