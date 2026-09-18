import { useEffect, useMemo, useState } from "react";
import { FiLock } from "react-icons/fi";
import AboutPage from "./AboutPage";
import type { Lang } from "../i18n";
import ShortcutRecorder from "../shortcuts/ShortcutRecorder";
import FontSelect, { FontOption } from "../components/FontSelect";
import {
  DEFAULT_SHORTCUTS,
  SHORTCUT_ACTIONS,
  loadShortcuts,
  saveShortcuts,
  ShortcutAction,
} from "../shortcuts/shortcuts";
import {
  allThemes,
  applyThemeObject,
  cloneAsCustom,
  getUserThemes,
  persistUserThemes,
  Theme,
  ThemeFonts,
} from "../theme/themes";
import { Settings } from "../settings";
import { fmt, t } from "../i18n";
import { useI18n } from "../i18n-context";
import { loadWorkspaces, persistWorkspaces, sha256, LOCK_SALT } from "../workspaces";

// WYSIWYG font lists — every option renders in its own face.
const UI_FONTS: FontOption[] = [
  { label: "系统默认（Geist）", css: "", fallback: "sans" },
  { label: "Geist Sans（内置）", css: '"Geist Sans"', fallback: "sans" },
  { label: "Noto Sans CJK SC", css: '"Noto Sans CJK SC"', fallback: "sans" },
  { label: "Noto Sans", css: '"Noto Sans"', fallback: "sans" },
  { label: "WenQuanYi Micro Hei", css: '"WenQuanYi Micro Hei"', fallback: "sans" },
  { label: "Source Han Sans SC", css: '"Source Han Sans SC"', fallback: "sans" },
  { label: "Ubuntu", css: "Ubuntu", fallback: "sans" },
  { label: "DejaVu Sans", css: '"DejaVu Sans"', fallback: "sans" },
  { label: "Inter", css: "Inter", fallback: "sans" },
  { label: "Roboto", css: "Roboto", fallback: "sans" },
  { label: "Microsoft YaHei", css: '"Microsoft YaHei"', fallback: "sans" },
  { label: "PingFang SC", css: '"PingFang SC"', fallback: "sans" },
];
const MONO_FONTS: FontOption[] = [
  { label: "默认等宽", css: "", fallback: "mono" },
  { label: "JetBrains Mono", css: '"JetBrains Mono"', fallback: "mono" },
  { label: "Cascadia Code", css: '"Cascadia Code"', fallback: "mono" },
  { label: "Fira Code", css: '"Fira Code"', fallback: "mono" },
  { label: "Source Code Pro", css: '"Source Code Pro"', fallback: "mono" },
  { label: "Hack", css: "Hack", fallback: "mono" },
  { label: "DejaVu Sans Mono", css: '"DejaVu Sans Mono"', fallback: "mono" },
  { label: "Ubuntu Mono", css: '"Ubuntu Mono"', fallback: "mono" },
  { label: "Liberation Mono", css: '"Liberation Mono"', fallback: "mono" },
  { label: "Noto Sans Mono", css: '"Noto Sans Mono"', fallback: "mono" },
  { label: "Consolas", css: "Consolas", fallback: "mono" },
  { label: "Menlo", css: "Menlo", fallback: "mono" },
  { label: "Sarasa Mono SC", css: '"Sarasa Mono SC"', fallback: "mono" },
  { label: "Courier New", css: '"Courier New"', fallback: "mono" },
];

// Left category tabs (egui parity: 通用/主题/快捷键/锁定).
const CATEGORIES = [
  { id: "general", label: "通用" },
  { id: "appearance", label: "外观" },
  { id: "shortcuts", label: "快捷键" },
  { id: "lock", label: "锁定" },
  { id: "about", label: "关于" },
] as const;

export default function SettingsPage({
  settings,
  onSettings,
  themeId,
  onTheme,
  lang,
}: {
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
  themeId: string;
  onTheme: (id: string) => void;
  lang: string;
}) {
  const T = useI18n();
  const t0 = t(lang as Lang);
  const CAT_LABEL: Record<(typeof CATEGORIES)[number]["id"], string> = {
    general: T.sGeneral, appearance: t0.appearance, shortcuts: t0.shortcuts, lock: t0.lockSection, about: t0.about,
  };
  const [cat, setCat] = useState<(typeof CATEGORIES)[number]["id"]>("general");
  const [shells, setShells] = useState<string[]>([]);
  useEffect(() => {
    import("@tauri-apps/api/core").then((m) => m.invoke<string[]>("list_shells")).then(setShells).catch(() => {});
  }, []);
  // Locally installed font families (fontconfig) — merged into the pickers.
  const [localFonts, setLocalFonts] = useState<string[]>([]);
  useEffect(() => {
    import("@tauri-apps/api/core").then((m) => m.invoke<string[]>("list_fonts")).then(setLocalFonts).catch(() => {});
  }, []);
  const uiFontOptions: FontOption[] = [
    { ...UI_FONTS[0], label: T.sSystemDefault },
    ...UI_FONTS.slice(1),
    ...localFonts
      .filter((f) => !UI_FONTS.some((o) => o.label === f))
      .map((f) => ({ label: f, css: `"${f}"`, fallback: "sans" as const })),
  ];
  const monoFontOptions: FontOption[] = [
    { ...MONO_FONTS[0], label: T.sMonoDefault },
    ...MONO_FONTS.slice(1),
    ...localFonts
      .filter((f) => !MONO_FONTS.some((o) => o.label === f))
      .map((f) => ({ label: f, css: `"${f}"`, fallback: "mono" as const })),
  ];

  // ---- theme editor -----------------------------------------------------
  const [userThemesV, setUserThemesV] = useState(0);
  const themes = useMemo(() => allThemes(), [userThemesV]);
  const [draft, setDraft] = useState<Theme | null>(null);
  const patchColor = (k: string, v: string) => {
    if (!draft) return;
    const next = { ...draft, colors: { ...draft.colors, [k]: v } };
    setDraft(next);
    applyThemeObject(next); // live preview
  };
  const patchTerm = (k: string, v: string) => {
    if (!draft?.term) return;
    const next = { ...draft, term: { ...draft.term, [k]: v } };
    setDraft(next);
    applyThemeObject(next);
  };
  const patchAnsi = (i: number, v: string) => {
    if (!draft?.term) return;
    const ansi = [...draft.term.ansi];
    ansi[i] = v;
    const next = { ...draft, term: { ...draft.term, ansi } };
    setDraft(next);
    applyThemeObject(next);
  };
  const saveDraft = () => {
    if (!draft) return;
    const list = getUserThemes();
    const idx = list.findIndex((t) => t.id === draft.id);
    if (idx >= 0) list[idx] = draft;
    else list.push(draft);
    persistUserThemes(list);
    setUserThemesV((v) => v + 1);
    window.dispatchEvent(new CustomEvent("opennex-themes-changed"));
    onTheme(draft.id);
    setDraft(null);
  };
  const deleteTheme = (id: string) => {
    persistUserThemes(getUserThemes().filter((t) => t.id !== id));
    setUserThemesV((v) => v + 1);
    window.dispatchEvent(new CustomEvent("opennex-themes-changed"));
    if (themeId === id) onTheme("midnight");
  };
  /** Edit a theme (copy for presets), with a font pack seeded from the
   * current effective settings. */
  const openEditor = (t: Theme) => {
    const base =
      t.id.startsWith("user-")
        ? { ...t, colors: { ...t.colors }, term: { ...t.term!, ansi: [...t.term!.ansi] } }
        : cloneAsCustom(t);
    setDraft({
      ...base,
      font:
        base.font ?? {
          uiFont: settings.uiFont,
          uiFontSize: settings.uiFontSize,
          termFont: settings.termFont,
          termFontSize: settings.fontSize,
        },
    });
  };
  const patchFont = (patch: Partial<ThemeFonts>) => {
    if (!draft) return;
    const next = { ...draft, font: { ...(draft.font ?? { uiFont: "", uiFontSize: 13, termFont: "", termFontSize: 14 }), ...patch } };
    setDraft(next);
    applyThemeObject(next);
  };

  // ---- shortcuts --------------------------------------------------------
  const [shortcuts, setShortcuts] = useState(loadShortcuts);
  const updateShortcut = (action: ShortcutAction, binding: string) => {
    const next = { ...shortcuts, [action]: binding };
    setShortcuts(next);
    saveShortcuts(next);
  };

  // ---- lock management -------------------------------------------------
  const workspaces = loadWorkspaces();
  const [lockPwd, setLockPwd] = useState("");
  const [lockMsg, setLockMsg] = useState<string | null>(null);
  const setAllLockPasswords = async () => {
    if (lockPwd.length < 4) return setLockMsg(t0.pwdShort);
    const hash = await sha256(LOCK_SALT + lockPwd);
    const list = loadWorkspaces();
    persistWorkspaces(list.map((w) => ({ ...w, lockHash: hash })));
    setLockPwd("");
    setLockMsg(fmt(T.sLockSetN, { n: list.length }));
  };
  const clearAllLocks = () => {
    persistWorkspaces(loadWorkspaces().map((w) => ({ ...w, locked: false, lockHash: undefined })));
    setLockMsg(t0.lockCleared);
  };

  return (
    <div className="settings-controls flex h-full">
      {/* 左侧分类栏 */}
      <div className="w-32 shrink-0 border-r border-[var(--border)] bg-[var(--bg-panel)] py-3">
        {CATEGORIES.map((c) => (
          <div
            key={c.id}
            onClick={() => setCat(c.id)}
            className={`nav-item !rounded-none !px-3 ${cat === c.id ? "active" : ""}`}
          >
            <span className="text-[12.5px]">{CAT_LABEL[c.id]}</span>
          </div>
        ))}
      </div>

      {/* 右侧内容 */}
      <div className="min-w-0 flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-[480px] space-y-6">
          {cat === "general" && (
            <>
              <section>
                <h2 className="mb-3 text-[15px] font-semibold">{T.sCommands}</h2>
                <div className="space-y-3 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                  <div className="flex items-center justify-between gap-3">
                    <span className="text-[12px] text-[var(--text-dim)]">{T.sHistoryCap}</span>
                    <select
                      value={settings.historyCap}
                      onChange={(e) => onSettings({ historyCap: Number(e.target.value) })}
                      className="rounded-md border border-[var(--border)] px-2 py-1 text-[12px] outline-none"
                      style={{ backgroundColor: "var(--bg-elevated)", color: "var(--text)" }}
                    >
                      {[100, 300, 500, 1000, 2000, 5000].map((n) => (
                        <option key={n} value={n} style={{ backgroundColor: "var(--bg-elevated)", color: "var(--text)" }}>
                          {fmt(T.sEntries, { n })}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>
              </section>

              <section>
                <h2 className="mb-3 text-[15px] font-semibold">{t0.defaultShellSection}</h2>
                <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                  {shells.length === 0 ? (
                    <div className="text-[12px] text-[var(--text-faint)]">{T.sShellsReadFail}</div>
                  ) : (
                    <div className="space-y-1.5">
                      {shells.map((s) => (
                        <label key={s} className="flex cursor-pointer items-center gap-2.5 text-[13px]">
                          <input
                            type="radio"
                            name="shell"
                            checked={settings.shell === s}
                            onChange={() => onSettings({ shell: s })}
                            className="accent-[var(--accent)]"
                          />
                          <span className="font-mono text-[12px]">{s}</span>
                        </label>
                      ))}
                    </div>
                  )}
                </div>
              </section>
            </>
          )}

          {cat === "appearance" && (
            <>
              <section>
                <h2 className="mb-3 text-[15px] font-semibold">{T.sFonts}</h2>
                <div className="space-y-3 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                  <label className="flex cursor-pointer items-center justify-between gap-3">
                    <span className="text-[12px] text-[var(--text-dim)]">
                      {T.sUseThemeFont}（{T.sUseThemeFontHint}）
                    </span>
                    <input
                      type="checkbox"
                      checked={settings.useThemeFont}
                      onChange={(e) => onSettings({ useThemeFont: e.target.checked })}
                      className="accent-[var(--accent)]"
                    />
                  </label>
                  <div className={settings.useThemeFont ? "pointer-events-none space-y-3 opacity-40" : "space-y-3"}>
                    <div className="flex items-center justify-between gap-3">
                      <span className="shrink-0 text-[12px] text-[var(--text-dim)]">{T.sUiFont}</span>
                      <FontSelect
                        value={settings.uiFont}
                        options={uiFontOptions}
                        onChange={(v) => onSettings({ uiFont: v })}
                        disabled={settings.useThemeFont}
                      />
                    </div>
                    <div className="flex items-center justify-between gap-3">
                      <span className="shrink-0 text-[12px] text-[var(--text-dim)]">
                        {T.sUiFontSize} · {settings.uiFontSize}px
                      </span>
                      <input
                        type="range"
                        min={11}
                        max={18}
                        disabled={settings.useThemeFont}
                        value={settings.uiFontSize}
                        onChange={(e) => onSettings({ uiFontSize: Number(e.target.value) })}
                        className="w-40 accent-[var(--accent)] disabled:opacity-40"
                      />
                    </div>
                    <div className="flex items-center justify-between gap-3">
                      <span className="shrink-0 text-[12px] text-[var(--text-dim)]">{T.sTermFont}</span>
                      <FontSelect
                        value={settings.termFont}
                        options={monoFontOptions}
                        onChange={(v) => onSettings({ termFont: v })}
                        disabled={settings.useThemeFont}
                      />
                    </div>
                    <div className="flex items-center justify-between gap-3">
                      <span className="shrink-0 text-[12px] text-[var(--text-dim)]">
                        {T.sTermFontSize} · {settings.fontSize}px
                      </span>
                      <input
                        type="range"
                        min={10}
                        max={24}
                        disabled={settings.useThemeFont}
                        value={settings.fontSize}
                        onChange={(e) => onSettings({ fontSize: Number(e.target.value) })}
                        className="w-40 accent-[var(--accent)] disabled:opacity-40"
                      />
                    </div>
                  </div>
                  <div className="text-[11px] leading-relaxed text-[var(--text-faint)]">
                    {T.sFontsHint}
                  </div>
                </div>
              </section>

              <section>
                <h2 className="mb-3 text-[15px] font-semibold">{t0.theme}</h2>
                <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                  <div className="mb-2 flex items-center justify-between text-[12px] text-[var(--text-dim)]">
                    <span>{T.sThemeManage}</span>
                    <button
                      className="rounded-md border border-[var(--border)] px-2 py-0.5 text-[11px] hover:border-[var(--accent)] hover:text-[var(--accent)]"
                      onClick={() => openEditor(themes.find((t) => t.id === themeId) ?? themes[0])}
                    >
                      {T.sThemeNew}
                    </button>
                  </div>
                  <div className="grid grid-cols-3 gap-2">
                    {themes.map((t) => (
                      <div
                        key={t.id}
                        onClick={() => onTheme(t.id)}
                        className={`group flex cursor-pointer items-center gap-2 rounded-md border px-2.5 py-2 text-[12px] transition-colors ${
                          themeId === t.id
                            ? "border-[var(--accent)] bg-[var(--bg-hover)]"
                            : "border-[var(--border)] hover:border-[var(--text-faint)]"
                        }`}
                      >
                        <span
                          className="h-4 w-4 shrink-0 rounded-full border border-[var(--border)]"
                          style={{ background: t.colors.bg }}
                        />
                        <span className="min-w-0 flex-1 truncate">{t.name}</span>
                        <button
                          className="shrink-0 text-[var(--text-faint)] opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--accent)]"
                          title={t.id.startsWith("user-") ? T.sThemeEdit : T.sThemeEditCopy}
                          onClick={(e) => {
                            e.stopPropagation();
                            openEditor(t);
                          }}
                        >
                          ✎
                        </button>
                        {t.id.startsWith("user-") && (
                          <button
                            className="shrink-0 text-[var(--text-faint)] opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                            title={t0.clear === "清除" ? "删除" : t0.clear}
                            onClick={(e) => { e.stopPropagation(); deleteTheme(t.id); }}
                          >
                            ✕
                          </button>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </section>

              {draft && (
                <section>
                  <h2 className="mb-3 text-[15px] font-semibold">{T.sEditTheme}（{T.sLivePreview}）</h2>
                  <div className="space-y-4 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                    <div className="flex items-center gap-2">
                      <input
                        value={draft.name}
                        onChange={(e) => setDraft({ ...draft, name: e.target.value })}
                        className="dialog-input !w-48"
                        placeholder={T.sThemeNamePh}
                      />
                      <button
                        className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
                        onClick={saveDraft}
                      >
                        {t0.save}
                      </button>
                      <button
                        className="rounded-md border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]"
                        onClick={() => {
                          setDraft(null);
                          onTheme(themeId); // restore the applied theme
                        }}
                      >
                        {t0.cancel}
                      </button>
                    </div>

                    <div>
                      <div className="mb-1.5 text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{T.sUi}</div>
                      <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
                        {([
                          ["bg", "背景"], ["bgPanel", "面板"], ["bgElevated", "浮层"],
                          ["bgHover", "悬停"], ["bgActive", "激活"], ["border", "边框"],
                          ["text", "文本"], ["textDim", "次要文本"], ["textFaint", "弱文本"],
                          ["accent", "强调"], ["accentDim", "强调底"], ["danger", "危险"], ["success", "成功"],
                        ] as const).map(([k, label]) => (
                          <label key={k} className="flex items-center gap-2 text-[11px] text-[var(--text-dim)]">
                            <span className="w-14 shrink-0">{label}</span>
                            <input
                              type="color"
                              value={draft.colors[k]}
                              onChange={(e) => patchColor(k, e.target.value)}
                              className="h-6 w-9 cursor-pointer rounded border border-[var(--border)] bg-transparent"
                            />
                            <span className="min-w-0 flex-1 truncate font-mono text-[10px] text-[var(--text-faint)]">{draft.colors[k]}</span>
                          </label>
                        ))}
                      </div>
                    </div>

                    <div>
                      <div className="mb-1.5 text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{t0.terminal}</div>
                      <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
                        {([
                          ["background", "背景"], ["foreground", "前景"],
                          ["cursor", "光标"], ["selection", "选区"],
                        ] as const).map(([k, label]) => (
                          <label key={k} className="flex items-center gap-2 text-[11px] text-[var(--text-dim)]">
                            <span className="w-14 shrink-0">{label}</span>
                            <input
                              type="color"
                              value={draft.term!.ansi ? (draft.term as any)[k] || "#000000" : "#000000"}
                              onChange={(e) => patchTerm(k, e.target.value)}
                              className="h-6 w-9 cursor-pointer rounded border border-[var(--border)] bg-transparent"
                            />
                            <span className="min-w-0 flex-1 truncate font-mono text-[10px] text-[var(--text-faint)]">{(draft.term as any)[k]}</span>
                          </label>
                        ))}
                      </div>
                    </div>

                    <div>
                      <div className="mb-1.5 text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{T.sFontWithTheme}</div>
                      <div className="space-y-2">
                        <div className="flex items-center justify-between gap-3">
                          <span className="w-14 shrink-0 text-[11px] text-[var(--text-dim)]">界面</span>
                          <FontSelect
                            value={draft.font?.uiFont ?? ""}
                            options={uiFontOptions}
                            onChange={(v) => patchFont({ uiFont: v })}
                          />
                          <input
                            type="number"
                            min={11}
                            max={18}
                            value={draft.font?.uiFontSize ?? 13}
                            onChange={(e) => patchFont({ uiFontSize: Number(e.target.value) || 13 })}
                            className="w-16 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1 text-[12px] text-[var(--text)]"
                          />
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span className="w-14 shrink-0 text-[11px] text-[var(--text-dim)]">终端</span>
                          <FontSelect
                            value={draft.font?.termFont ?? ""}
                            options={monoFontOptions}
                            onChange={(v) => patchFont({ termFont: v })}
                          />
                          <input
                            type="number"
                            min={10}
                            max={24}
                            value={draft.font?.termFontSize ?? 14}
                            onChange={(e) => patchFont({ termFontSize: Number(e.target.value) || 14 })}
                            className="w-16 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1 text-[12px] text-[var(--text)]"
                          />
                        </div>
                      </div>
                    </div>

                    <div>
                      <div className="mb-1.5 text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{T.sAnsiPalette}</div>
                      <div className="grid grid-cols-4 gap-1.5">
                        {draft.term!.ansi.map((c, i) => (
                          <label key={i} className="flex items-center gap-1.5 text-[10px] text-[var(--text-dim)]">
                            <input
                              type="color"
                              value={c}
                              onChange={(e) => patchAnsi(i, e.target.value)}
                              className="h-6 w-8 cursor-pointer rounded border border-[var(--border)] bg-transparent"
                            />
                            <span className="min-w-0 flex-1 truncate font-mono">{i}</span>
                          </label>
                        ))}
                      </div>
                    </div>
                  </div>
                </section>
              )}
            </>
          )}

          {cat === "shortcuts" && (
            <section>
              <h2 className="mb-3 text-[15px] font-semibold">{t0.shortcuts}</h2>
              <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                <p className="mb-3 text-[11px] text-[var(--text-faint)]">
                  {lang === "zh" || lang === "zh-TW"
                    ? "点击快捷键名称后按下新按键即可修改"
                    : "Click a binding, then press new keys to rebind"}
                </p>
                <div className="space-y-2">
                  {SHORTCUT_ACTIONS.map((a) => (
                    <div key={a.id} className="flex items-center justify-between gap-3">
                      <span className="text-[12px] text-[var(--text-dim)]">
                        {lang === "zh" || lang === "zh-TW" ? a.labelZh : a.labelEn}
                      </span>
                      <ShortcutRecorder
                        binding={shortcuts[a.id]}
                        onChange={(b) => updateShortcut(a.id, b)}
                      />
                    </div>
                  ))}
                </div>
                <button
                  className="mt-3 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)] hover:text-[var(--text)]"
                  onClick={() => {
                    // 恢复默认必须显式落盘：loadShortcuts() 会先读到旧存档。
                    const defaults = { ...DEFAULT_SHORTCUTS };
                    setShortcuts(defaults);
                    saveShortcuts(defaults);
                  }}
                >
                  {lang === "zh" || lang === "zh-TW" ? "恢复默认按键" : "Reset to defaults"}
                </button>
              </div>
            </section>
          )}

          {cat === "lock" && (
            <section>
              <h2 className="mb-3 text-[15px] font-semibold">{t0.lockSection}</h2>
              <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                <div className="mb-2 flex items-center gap-1.5 text-[12px] text-[var(--text-dim)]">
                  <FiLock size={13} /> {t0.lockAllHint}
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="password"
                    value={lockPwd}
                    onChange={(e) => { setLockPwd(e.target.value); setLockMsg(null); }}
                    placeholder={T.sPwdPh}
                    className="dialog-input !w-52"
                  />
                  <button className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
                    onClick={setAllLockPasswords}>{t0.set}</button>
                  <button className="rounded-md border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--danger)]"
                    onClick={clearAllLocks}>{t0.clear}</button>
                </div>
                {lockMsg && <div className="mt-2 text-[11px] text-[var(--success)]">{lockMsg}</div>}
              </div>
            </section>
          )}

          {cat === "about" && (
            <section>
              <h2 className="mb-3 text-[15px] font-semibold">{t0.about}</h2>
              <AboutPage lang={lang as Lang} embedded />
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
