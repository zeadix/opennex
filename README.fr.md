[简体中文](README.md) | [English](README.en.md) | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | **Français** | [日本語](README.ja.md) | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **gestionnaire de terminaux IA à fenêtres empilées** — transforme n'importe quel outil de codage IA ou d'agent en station de travail multi-instances pilotable à distance.

| | |
|---|---|
| **1000+** | fenêtres actives simultanées |
| **20+** | langues d'interface |
| **3** | plates-formes natives (Linux / Windows / macOS) |

## Fonctionnalités clés

**✦ Exécution IA**
L'IA planifie et exécute les tâches du terminal — interruptible et suspensible à tout moment.

**🌐 Contrôle à distance WAN**
Prenez le contrôle de sessions depuis téléphone, tablette ou PC au-delà du LAN.

**▦ Disposition flexible**
Organisez et empilez librement les fenêtres ; dispositions sauvegardables et rechargeables.

**↩ Mémoire des commandes**
Historique illimité avec rappel en une touche.

**★ Favoris**
Vos commandes fréquentes accessibles en un clic.

**◈ Thèmes**
Thèmes personnalisés pour un terminal à votre image.

**🔒 Chiffrement des espaces**
Verrouillage par mot de passe maître et espaces chiffrés.

**⇄ Contrôle à distance LAN**
Pilotez les sessions depuis tout appareil du même réseau local.

**◉ Détection actif/occupé**
Statut en direct de chaque espace de travail.

**⇅ Hôtes SSH**
Gestion intégrée des hôtes SSH, accès en un clic.

**⚡ Surveillance des performances**
Ressources fenêtres et système en temps réel.

**⟲ Mémoire des espaces**
L'état est sauvegardé et restauré au relancement.

**🖥 Infos système**
Informations et ressources de la machine en temps réel.

**⤺ Restauration des chemins**
Les répertoires de travail sont restaurés après redémarrage.

**🌐 Multilingue**
Plus de 20 langues d'interface, changement en un clic.

## Plates-formes prises en charge

| Plate-forme | Minimum | Installation |
|------|---------|---------|
| Windows | Windows 10+ | Installateur NSIS / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

Téléchargez depuis le [site](https://opennex.zeadix.com) ou [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) ; mises à jour intégrées vérifiées par signature.

## Développement & tests

```bash
cd tauri-app

# installer les dépendances front (première fois)
npm install

# mode dev : rechargement à chaud vite + build Rust debug (ou ./dev.sh)
npm run tauri dev

# vérification des types / build de production
npx tsc --noEmit
npm run build

# paquet (plate-forme courante)
npm run tauri build
```

### CI/CD

Pousser un tag `v*.*.*` déclenche GitHub Actions : builds multi-plates-formes (NSIS+MSI / DMG / DEB+AppImage), paquets de mise à jour signés minisign, flux de mise à jour `tauri/latest.json` vers R2, GitHub Release et reconstruction du site :

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Structure du projet

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
