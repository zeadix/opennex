// WebKit 引擎端到端测试：验证补全面板跟随终端输入光标。
import { webkit } from "playwright";

// WPE 启动偶发库加载竞态：重试若干次
let b = null;
for (let i = 0; i < 6 && !b; i++) {
  try {
    b = await webkit.launch({ headless: true });
  } catch (e) {
    console.log(`launch retry ${i + 1}:`, String(e).slice(0, 120));
    await new Promise((r) => setTimeout(r, 800));
  }
}
if (!b) throw new Error("webkit launch failed after retries");
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1.5 });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19999 });
      if (cmd === "get_history")
        return Promise.resolve([
          { id: 1, cmd: "docker ps -a", hits: 5 },
          { id: 2, cmd: "git status", hits: 3 },
          { id: 3, cmd: "kubectl get pods", hits: 2 },
        ]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      if (cmd === "list_path_commands")
        return Promise.resolve(["docker", "docker-compose", "kubectl", "ls", "git", "grep", "curl"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {},
    postMessage() {},
  };
  localStorage.setItem(
    "opennex-settings",
    JSON.stringify({ autoMatch: true, followCursor: true, suggestSource: "auto" }),
  );
  localStorage.removeItem("opennex-suggest-pos");
  localStorage.removeItem("opennex-palette-pos");
});
const page = await ctx.newPage();
page.on("pageerror", (e) => console.log("PAGEERROR:", String(e).slice(0, 200)));
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(8000); // pane 布局等待(最多5s) + shell 启动 + 提示符

await page.locator(".xterm-screen").first().click({ position: { x: 60, y: 20 } });
await page.keyboard.type("gi");
await page.waitForTimeout(700);

const out = await page.evaluate(() => {
  const panel = document.querySelector('div[class*="z-[6000]"]');
  const pr = panel?.getBoundingClientRect();
  const host = panel?.parentElement;
  const hostR = host?.getBoundingClientRect();
  let caret = null;
  host?.querySelectorAll(".xterm-rows > div").forEach((row) => {
    if (caret === null && (row.textContent ?? "").includes("$")) {
      const rng = document.createRange();
      rng.selectNodeContents(row);
      const rr = rng.getBoundingClientRect();
      caret = {
        right: Math.round(rr.right),
        top: Math.round(rr.top),
        bottom: Math.round(rr.bottom),
        text: row.textContent,
      };
    }
  });
  return {
    engine: navigator.userAgent.slice(0, 70),
    panelOpen: !!panel,
    panelHead: panel?.textContent?.slice(0, 26) ?? null,
    panel: pr
      ? { left: Math.round(pr.left), top: Math.round(pr.top), right: Math.round(pr.right), bottom: Math.round(pr.bottom) }
      : null,
    caret,
    panelInsidePane: !!(panel && hostR && pr && pr.left >= hostR.left - 1 && pr.right <= hostR.right + 1),
    belowCaretLine:
      pr && caret ? pr.top >= caret.bottom - 3 && pr.top <= caret.bottom + 30 : null,
  };
});
console.log(JSON.stringify(out, null, 2));
await b.close();
