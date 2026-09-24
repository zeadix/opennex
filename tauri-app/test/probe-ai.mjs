import { webkit } from "playwright";

const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 } });
await ctx.addInitScript(() => {
  const myShim = {
    invoke(cmd, args) {
      window.__cmds.push(String(cmd));
      if (cmd === "start_terminal")
        return Promise.resolve({ session: "s1", wsPort: 19999 });
      if (cmd === "ai_chat") return Promise.resolve("测试回复甲乙丙");
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  window.__myShim = myShim;
  window.__TAURI_INTERNALS__ = myShim;
  window.__cmds = [];
  localStorage.setItem("opennex-lang", "zh");
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: true }));
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(8000);
const out = await page.evaluate(() => {
  const same = window.__TAURI_INTERNALS__ === window.__myShim;
  const keys = Object.keys(window.__TAURI_INTERNALS__ ?? {});
  return { same, keys, cmdsLen: window.__cmds.length };
});
console.log(JSON.stringify(out, null, 2));
await b.close();
