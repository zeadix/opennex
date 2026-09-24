// 复现: 双垂直滚动条 + 水平滚动条闪烁
import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 900, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19999 });
      if (cmd === "get_history") return Promise.resolve([]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      if (cmd === "list_path_commands") return Promise.resolve(["docker", "git"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: true, followCursor: true }));
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(9000);
const info = await page.evaluate(() => {
  const doc = document.documentElement;
  const hosts = Array.from(document.querySelectorAll("[data-term-slot]"));
  return {
    bodyScrollW: doc.scrollWidth,
    bodyClientW: doc.clientWidth,
    bodyOverflowX: doc.scrollWidth > doc.clientWidth,
    bodyOverflowY: doc.scrollHeight > doc.clientHeight,
    hosts: hosts.map((h) => {
      const r = h.getBoundingClientRect();
      const screen = h.querySelector(".xterm-screen")?.getBoundingClientRect();
      const viewport = h.querySelector(".xterm-viewport")?.getBoundingClientRect();
      return {
        slot: h.getAttribute("data-term-slot"),
        host: { w: Math.round(r.width), h: Math.round(r.height) },
        screen: screen ? { w: Math.round(screen.width), left: Math.round(screen.left) } : null,
        viewport: viewport ? { w: Math.round(viewport.width), sw: viewport.scrollWidth, cw: viewport.clientWidth } : null,
      };
    }),
  };
});
console.log(JSON.stringify(info, null, 2));
await page.screenshot({ path: "/tmp/scrollbars.png" });
await b.close();
