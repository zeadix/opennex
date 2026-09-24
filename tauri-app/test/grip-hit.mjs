import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19999 });
      if (cmd === "get_history") return Promise.resolve([{ id: 1, cmd: "docker ps -a", hits: 5 }]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      if (cmd === "list_path_commands") return Promise.resolve(["docker", "git"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: true, followCursor: false }));
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(8000);
await page.locator(".xterm-screen").first().click({ position: { x: 60, y: 20 } });
await page.keyboard.type("gi");
await page.waitForTimeout(300);
await page.keyboard.press("Escape");
await page.waitForTimeout(150);
await page.keyboard.press("Alt");
await page.waitForTimeout(1000);
const out = await page.evaluate(() => {
  const grip = document.querySelector("body > div.animate-fade-up.fixed .overlay-grip");
  const r = grip?.getBoundingClientRect();
  // 命中测试: grip 中心点处实际是什么元素
  const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
  return {
    gripRect: r ? { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) } : null,
    hitTag: el?.tagName,
    hitCls: String(el?.className ?? "").slice(0, 60),
  };
});
console.log(JSON.stringify(out, null, 2));
await b.close();
