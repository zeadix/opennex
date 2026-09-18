import assert from "node:assert/strict";
import { test } from "node:test";
import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

async function fixture() {
  const dir = await mkdtemp(join(tmpdir(), "opennex-updates-"));
  await build({
    stdin: {
      contents: 'export * from "./src/updates";',
      resolveDir: new URL("../", import.meta.url).pathname,
    },
    outfile: join(dir, "updates.mjs"), bundle: true, platform: "node", format: "esm",
    plugins: [{ name: "mock-ipc", setup(b) {
      b.onResolve({ filter: /terminal\/tauri$/ }, () => ({ path: "ipc", namespace: "test" }));
      b.onLoad({ filter: /.*/, namespace: "test" }, () => ({ contents: 'export const invoke = (...args) => globalThis.updateInvoke(...args);' }));
    } }],
  });
  const api = await import(pathToFileURL(join(dir, "updates.mjs")));
  await rm(dir, { recursive: true });
  return api;
}

const result = {
  current: "0.1.0", latest: "0.1.55", channel: "egui", updateAvailable: false, canInstall: false,
  changes: ["最新版本日志"], changesEn: ["Latest notes"], currentChanges: ["当前版本日志"], currentChangesEn: [],
};

test("startup check runs once and overlapping checks share one request", async () => {
  const api = await fixture();
  const calls = [];
  let resolve;
  globalThis.updateInvoke = (command) => {
    calls.push(command);
    return command === "get_app_info" ? Promise.resolve({ current: "0.1.0" }) : new Promise(r => { resolve = r; });
  };
  api.startUpdateCheck();
  api.startUpdateCheck();
  const first = api.checkForUpdates();
  assert.equal(first, api.checkForUpdates());
  assert.deepEqual(calls, ["get_app_info", "check_update"]);
  resolve(result);
  await first;
});

test("failed check can be retried", async () => {
  const api = await fixture();
  let calls = 0;
  globalThis.updateInvoke = () => ++calls === 1 ? Promise.reject(new Error("offline")) : Promise.resolve(result);
  await api.checkForUpdates();
  await api.checkForUpdates();
  assert.equal(calls, 2);
});

test("English notes fall back to Chinese without borrowing another release", async () => {
  const api = await fixture();
  assert.deepEqual(api.releaseNotes(["中文"], ["", "  "], "en"), ["中文"]);
  assert.deepEqual(api.releaseNotes(["中文"], ["English"], "en"), ["English"]);
  assert.deepEqual(api.releaseNotes([], [], "zh"), []);
});
