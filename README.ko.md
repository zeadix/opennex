[简体中文](README.md) | [English](README.en.md) | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [日本語](README.ja.md) | [Italiano](README.it.md) | **한국어** | [हिन्दी](README.hi.md)

# OpenNex

OpenNex — **다중 창 스택형 AI 터미널 매니저** — 어떤 AI 코딩·에이전트 도구든 다중 실행·원격 조작 가능한 스마트 워크스테이션으로 바꿔줍니다.

![전체 화면](docs/screenshots/panorama.png)

| | |
|---|---|
| **1000+** | 동시 실행 창 |
| **20+** | UI 언어 |
| **3** | 네이티브 플랫폼(Linux / Windows / macOS) |

## 스크린샷

**🤖 AI 어시스턴트 · 모델 설정**

![](docs/screenshots/ai-config.png)

모델 API와 컨텍스트 한도를 설정하면 AI 어시스턴트가 활성화됩니다.

**🤖 AI 어시스턴트 · 대화형 터미널 제어**

![](docs/screenshots/ai-chat-terminal.png)

API를 설정하면 대화로 AI에게 터미널 명령 실행이나 문제 해결을 지시할 수 있습니다. 언제든 개입 가능.

**🌐 원격 제어 도우미**

![](docs/screenshots/remote-control.png)

QR 코드를 스캔하거나 브라우저로 연결해 터미널을 원격 조작. 터미널에서 실행한 소프트웨어도 원격 제어 가능.

**🔒 작업공간 잠금**

![](docs/screenshots/workspace-lock.png)

자리를 비울 때 원클릭 잠금, 돌아오면 빠르게 해제.

**★ 명령 즐겨찾기**

![](docs/screenshots/command-favorites.png)

자주 쓰는 명령을 폴더로 모아 바로 호출.

**↩ 명령 자동 완성**

![](docs/screenshots/auto-complete.png)

입력 시 자동 매칭: 시스템 명령 + 이 터미널의 기록.

**◉ 활동 표시기**

![](docs/screenshots/idle-busy-indicator.png)

빨강 = 작업공간에 실행 중인 터미널 있음; 초록 = 모두 완료.

## 주요 기능

**✦ AI 자동 실행**
AI가 터미널 작업을 스스로 계획·실행합니다. 언제든 개입·일시정지 가능.

**🌐 WAN 원격 제어**
LAN 밖에서도 휴대폰·태블릿·PC로 터미널 세션을 원격 조작.

**▦ 유연한 창 레이아웃**
창을 자유롭게 배치·스택. 레이아웃 저장·복원으로 AI 도구 다중 실행도 질서있게.

**↩ 명령 기록**
입력한 명령을 무제한 기록, 단축키 한 번으로 불러옵니다.

**★ 즐겨찾기**
자주 쓰는 명령을 한 번의 클릭으로.

**◈ 테마**
커스텀 테마로 나만의 터미널 외관을.

**🔒 작업공간 암호화**
마스터 비밀번호 잠금과 작업공간 암호화로 민감한 환경도 안전.

**⇄ LAN 원격 제어**
같은 LAN 내 휴대폰·태블릿·PC에서 세션을 원격 조작.

**◉ 활동 감지**
작업공간별 사용 상태를 실시간 표시.

**⇅ SSH 호스트**
SSH 호스트 관리 내장, 원격 서버 원클릭 접속.

**⚡ 성능 모니터링**
창과 시스템 리소스를 실시간 확인.

**⟲ 작업공간 기억**
상태 자동 저장, 재시작 시 이전 상태 복원.

**🖥 시스템 정보**
호스트 시스템 정보와 리소스를 실시간 표시.

**⤺ 경로 복원**
재시작 후 각 창의 작업 디렉터리를 자동 복원.

**🌐 다국어**
20+ UI 언어를 클릭 한 번으로 전환.

## 지원 플랫폼

| 플랫폼 | 최소 요구 사항 | 설치 |
|------|---------|---------|
| Windows | Windows 10+ | NSIS 설치 프로그램 / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

모든 설치 패키지는 [공식 사이트](https://opennex.zeadix.com) 또는 [GitHub Releases](https://github.com/zeadix/opennex/releases/latest)에서. 서명 검증된 원클릭 업데이트 내장.

## 개발 및 테스트

```bash
cd tauri-app

# 프런트엔드 의존성 설치(최초)
npm install

# 개발 모드: vite 핫 리로드 + Rust 디버그 빌드(./dev.sh 가능)
npm run tauri dev

# 프런트엔드 타입 체크 / 프로덕션 빌드
npx tsc --noEmit
npm run build

# 패키징(현재 플랫폼)
npm run tauri build
```

### CI/CD

`v*.*.*` 형식의 태그를 push하면 GitHub Actions가 실행됩니다: 다중 플랫폼 패키징(NSIS+MSI / DMG / DEB+AppImage), minisign 서명 업데이트 패키지, 업데이트 피드 `tauri/latest.json`을 R2에 업로드, GitHub Release 발행 및 웹사이트 재빌드:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## 프로젝트 구조

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