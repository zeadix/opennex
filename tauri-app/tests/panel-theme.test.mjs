import assert from "node:assert/strict";
import { test } from "node:test";
import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const dir = await mkdtemp(join(tmpdir(), "opennex-panels-"));
await build({
  stdin: {
    contents: 'export * from "./src/theme/themes"; export * from "./src/dock/model"; export * from "./src/dock/mainLayout";',
    resolveDir: new URL("../", import.meta.url).pathname,
  },
  outfile: join(dir, "panels.mjs"), bundle: true, platform: "node", format: "esm",
});
const api = await import(pathToFileURL(join(dir, "panels.mjs")));
await rm(dir, { recursive: true });

test("every theme supplies public dock tokens with matching tab and content surfaces", () => {
  const properties = new Map();
  globalThis.document = { documentElement: { style: { setProperty: (k, v) => properties.set(k, v) } } };
  globalThis.window = { dispatchEvent() {} };
  for (const theme of api.THEMES) {
    api.applyThemeObject(theme);
    assert.equal(properties.get("--flexlayout-color-tab-content"), theme.colors.bgPanel);
    assert.equal(properties.get("--flexlayout-color-tab-selected-background"), theme.colors.bgPanel);
    assert.equal(properties.get("--flexlayout-color-text"), theme.colors.text);
    assert.equal(properties.get("--flexlayout-color-border-tab-content"), theme.colors.bgPanel);
  }
});

test("old saved panel titles migrate without changing identifiers or layout weights", () => {
  const json = api.mainModelFrom().toJson();
  const tabs = json.layout.children[1].children;
  const names = { "tab-ai": "AI助手", "tab-history": "历史指令", "tab-favorites": "指令收藏夹", "tab-ssh": "SSH 配置" };
  for (const id of Object.keys(names)) tabs.push({ type: "tab", id, name: "旧标题", component: id.slice(4) });
  const storage = new Map([["opennex-main-layout", JSON.stringify(json)]]);
  globalThis.localStorage = { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) };
  const model = api.loadMainModel();
  assert.equal(model.getNodeById("nav").getName(), "导航栏");
  for (const [id, name] of Object.entries(names)) assert.equal(model.getNodeById(id).getName(), name);
  assert.deepEqual(model.toJson().layout.children.map(n => n.weight), json.layout.children.map(n => n.weight));
});
