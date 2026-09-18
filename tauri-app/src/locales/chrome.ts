import type { Lang } from "../i18n";

const zh = {
  cTerminalArea: "终端工作区", cClose: "关闭", cUpdateLatest: "更新至最新版本",
  cTemplateCreate: "从模板创建工作空间", cTemplateEmpty: "暂无模板（右键工作空间可保存为模板）",
  cBroadcastStop: "关闭广播输入（当前工作空间）", cBroadcastStart: "开启广播输入（当前工作空间）",
  cBroadcastLeave: "退出广播组", cBroadcastJoin: "加入广播组", cRenameTerminal: "重命名终端", cCloseTerminal: "关闭终端",
  cDefaultWorkspace: "默认工作空间", cWorkspace: "工作空间", cResize: "调整大小", cMoreTabs: "更多标签", cTerminals: "个终端",
};
export const chromeMessages: Record<Lang, typeof zh> = {
  zh,
  "zh-TW": {
    cTerminalArea: "終端工作區", cClose: "關閉", cUpdateLatest: "更新至最新版本",
    cTemplateCreate: "從範本建立工作空間", cTemplateEmpty: "尚無範本（右鍵工作空間可儲存為範本）",
    cBroadcastStop: "關閉廣播輸入（目前工作空間）", cBroadcastStart: "開啟廣播輸入（目前工作空間）",
    cBroadcastLeave: "退出廣播群組", cBroadcastJoin: "加入廣播群組", cRenameTerminal: "重新命名終端", cCloseTerminal: "關閉終端",
    cDefaultWorkspace: "預設工作空間", cWorkspace: "工作空間", cResize: "調整大小", cMoreTabs: "更多分頁", cTerminals: "個終端",
  },
  en: {
    cTerminalArea: "Terminal workspace", cClose: "Close", cUpdateLatest: "Update to latest",
    cTemplateCreate: "Create workspace from template", cTemplateEmpty: "No templates yet (right-click a workspace to save one)",
    cBroadcastStop: "Stop broadcasting input (current workspace)", cBroadcastStart: "Broadcast input (current workspace)",
    cBroadcastLeave: "Leave broadcast group", cBroadcastJoin: "Join broadcast group", cRenameTerminal: "Rename terminal", cCloseTerminal: "Close terminal",
    cDefaultWorkspace: "Default workspace", cWorkspace: "Workspace", cResize: "Resize", cMoreTabs: "More tabs", cTerminals: " terminals",
  },
  de: {
    cTerminalArea: "Terminal-Arbeitsbereich", cClose: "Schließen", cUpdateLatest: "Auf neueste Version aktualisieren",
    cTemplateCreate: "Arbeitsbereich aus Vorlage erstellen", cTemplateEmpty: "Keine Vorlagen (Arbeitsbereich per Rechtsklick speichern)",
    cBroadcastStop: "Eingabeübertragung beenden (aktueller Arbeitsbereich)", cBroadcastStart: "Eingabe übertragen (aktueller Arbeitsbereich)",
    cBroadcastLeave: "Übertragungsgruppe verlassen", cBroadcastJoin: "Übertragungsgruppe beitreten", cRenameTerminal: "Terminal umbenennen", cCloseTerminal: "Terminal schließen",
    cDefaultWorkspace: "Standard-Arbeitsbereich", cWorkspace: "Arbeitsbereich", cResize: "Größe ändern", cMoreTabs: "Weitere Tabs", cTerminals: " Terminals",
  },
  fr: {
    cTerminalArea: "Espace terminal", cClose: "Fermer", cUpdateLatest: "Installer la dernière version",
    cTemplateCreate: "Créer un espace depuis un modèle", cTemplateEmpty: "Aucun modèle (clic droit sur un espace pour en enregistrer un)",
    cBroadcastStop: "Arrêter la diffusion (espace actuel)", cBroadcastStart: "Diffuser la saisie (espace actuel)",
    cBroadcastLeave: "Quitter le groupe de diffusion", cBroadcastJoin: "Rejoindre le groupe de diffusion", cRenameTerminal: "Renommer le terminal", cCloseTerminal: "Fermer le terminal",
    cDefaultWorkspace: "Espace par défaut", cWorkspace: "Espace de travail", cResize: "Redimensionner", cMoreTabs: "Autres onglets", cTerminals: " terminaux",
  },
  ja: {
    cTerminalArea: "ターミナル作業領域", cClose: "閉じる", cUpdateLatest: "最新版に更新",
    cTemplateCreate: "テンプレートからワークスペースを作成", cTemplateEmpty: "テンプレートなし（ワークスペースを右クリックして保存）",
    cBroadcastStop: "入力の一斉送信を停止（現在のワークスペース）", cBroadcastStart: "入力を一斉送信（現在のワークスペース）",
    cBroadcastLeave: "一斉送信グループから外す", cBroadcastJoin: "一斉送信グループに追加", cRenameTerminal: "ターミナル名を変更", cCloseTerminal: "ターミナルを閉じる",
    cDefaultWorkspace: "既定のワークスペース", cWorkspace: "ワークスペース", cResize: "サイズ変更", cMoreTabs: "その他のタブ", cTerminals: " ターミナル",
  },
  it: {
    cTerminalArea: "Area terminale", cClose: "Chiudi", cUpdateLatest: "Aggiorna all’ultima versione",
    cTemplateCreate: "Crea area da modello", cTemplateEmpty: "Nessun modello (clic destro su un’area per salvarne uno)",
    cBroadcastStop: "Interrompi trasmissione (area corrente)", cBroadcastStart: "Trasmetti input (area corrente)",
    cBroadcastLeave: "Esci dal gruppo di trasmissione", cBroadcastJoin: "Unisciti al gruppo di trasmissione", cRenameTerminal: "Rinomina terminale", cCloseTerminal: "Chiudi terminale",
    cDefaultWorkspace: "Area predefinita", cWorkspace: "Area di lavoro", cResize: "Ridimensiona", cMoreTabs: "Altre schede", cTerminals: " terminali",
  },
  ko: {
    cTerminalArea: "터미널 작업 영역", cClose: "닫기", cUpdateLatest: "최신 버전으로 업데이트",
    cTemplateCreate: "템플릿으로 작업 공간 만들기", cTemplateEmpty: "템플릿 없음 (작업 공간을 우클릭하여 저장)",
    cBroadcastStop: "입력 동시 전송 중지 (현재 작업 공간)", cBroadcastStart: "입력 동시 전송 (현재 작업 공간)",
    cBroadcastLeave: "동시 전송 그룹에서 나가기", cBroadcastJoin: "동시 전송 그룹에 참여", cRenameTerminal: "터미널 이름 변경", cCloseTerminal: "터미널 닫기",
    cDefaultWorkspace: "기본 작업 공간", cWorkspace: "작업 공간", cResize: "크기 조절", cMoreTabs: "더 많은 탭", cTerminals: "개 터미널",
  },
  hi: {
    cTerminalArea: "टर्मिनल कार्यक्षेत्र", cClose: "बंद करें", cUpdateLatest: "नवीनतम संस्करण में अपडेट करें",
    cTemplateCreate: "टेम्पलेट से कार्यक्षेत्र बनाएँ", cTemplateEmpty: "कोई टेम्पलेट नहीं (सहेजने के लिए कार्यक्षेत्र पर राइट-क्लिक करें)",
    cBroadcastStop: "इनपुट प्रसारण रोकें (वर्तमान कार्यक्षेत्र)", cBroadcastStart: "इनपुट प्रसारित करें (वर्तमान कार्यक्षेत्र)",
    cBroadcastLeave: "प्रसारण समूह छोड़ें", cBroadcastJoin: "प्रसारण समूह में जुड़ें", cRenameTerminal: "टर्मिनल का नाम बदलें", cCloseTerminal: "टर्मिनल बंद करें",
    cDefaultWorkspace: "डिफ़ॉल्ट कार्यक्षेत्र", cWorkspace: "कार्यक्षेत्र", cResize: "आकार बदलें", cMoreTabs: "और टैब", cTerminals: " टर्मिनल",
  },
};
