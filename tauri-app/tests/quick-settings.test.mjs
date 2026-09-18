import assert from "node:assert/strict";
import { test } from "node:test";
import { build } from "esbuild";
import { mkdtemp, rm, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const dir = await mkdtemp(join(tmpdir(), "opennex-quick-"));
await build({
  stdin: { contents: 'export { default as QuickSettings } from "./src/pages/QuickSettingsPage"; export * from "./src/settings"; export * from "./src/dock/model";', resolveDir: new URL("../", import.meta.url).pathname },
  outfile: join(dir, "quick.mjs"), bundle: true, platform: "node", format: "esm",
});
const api = await import(pathToFileURL(join(dir, "quick.mjs")));
await rm(dir, { recursive: true });

function inputs(node) {
  if (!node || typeof node !== "object") return [];
  if (Array.isArray(node)) return node.flatMap(inputs);
  return node.type === "input" ? [node] : inputs(node.props?.children);
}

test("quick settings toggles the existing persisted fields without resetting other settings", () => {
  const storage = new Map();
  globalThis.localStorage = { getItem: k => storage.get(k) ?? null, setItem: (k, v) => storage.set(k, v) };
  let settings = { ...api.defaultSettings, historyCap: 2000, fontSize: 18 };
  const toggles = inputs(api.QuickSettings({ settings, lang: "zh", onSettings: patch => { settings = { ...settings, ...patch }; api.saveSettings(settings); } }));
  assert.equal(toggles.length, 3);
  for (const input of toggles) input.props.onChange({ target: { checked: !input.props.checked } });
  const saved = api.loadSettings();
  assert.equal(saved.autoMatch, false);
  assert.equal(saved.copyOnSelect, false);
  assert.equal(saved.followCursor, true);
  assert.equal(saved.historyCap, 2000);
  assert.equal(saved.fontSize, 18);
});

test("saved main layout retains quick settings independently of terminal layout", () => {
  const json = api.mainModelFrom().toJson();
  json.layout.children[1].children.push({ type: "tab", id: "tab-quick-settings", name: "快捷设置", component: "quick-settings" });
  const restored = api.mainModelFrom(json);
  assert.equal(restored.getNodeById("tab-quick-settings").getComponent(), "quick-settings");
});

test("general settings no longer duplicates the three quick toggles", async () => {
  const source = await readFile(new URL("../src/pages/SettingsPage.tsx", import.meta.url), "utf8");
  for (const key of ["autoMatch", "copyOnSelect", "followCursor"]) assert.ok(!source.includes(`checked={settings.${key}}`));
  assert.ok(source.includes("settings.historyCap"));
});
