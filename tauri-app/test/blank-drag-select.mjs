// 实测: 拖选自动复制 —— 首点在内容下方空白处,拖到文字行,应产生选区
// (修复前 xterm 从未收到 mousedown,选区层恒空)。
import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1100, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19999 });
      if (cmd === "get_history") return Promise.resolve([]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: false, copyOnSelect: true }));
});
const page = await ctx.newPage();
let toasts = 0;
page.on("console", () => {});
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(9000);

// 几何:终端内容(最后一行)与下方空白
const g = await page.evaluate(() => {
  const screen = document.querySelector(".xterm-screen");
  const rows = document.querySelectorAll(".xterm-rows > div");
  const sr = screen.getBoundingClientRect();
  let last = 0;
  rows.forEach(r => { if ((r.textContent || "").trim()) last = Math.max(last, r.getBoundingClientRect().bottom); });
  return { sr: { l: sr.left, t: sr.top, r: sr.right, b: sr.bottom }, lastTextBottom: last, rowCount: rows.length };
});
console.log("geometry:", JSON.stringify(g));

// 首点:最后一行文字下方 60px 的空白(确保在 screen 外/宿主空白区)
const startY = Math.min(g.sr.b - 8, g.lastTextBottom + 60);
const startX = g.sr.l + 60;
// 拖到上面的文字行
await page.mouse.move(startX, startY);
await page.mouse.down();
await page.mouse.move(g.sr.l + 300, g.lastTextBottom - 25, { steps: 10 });
await page.mouse.up();
await page.waitForTimeout(400);

const sel = await page.evaluate(() => {
  const el = document.querySelector(".xterm-selection");
  const rects = el ? Array.from(el.querySelectorAll("div,span")).map(x => Math.round(x.getBoundingClientRect().width)) : [];
  return {
    layerExists: !!el,
    layerW: el ? Math.round(el.getBoundingClientRect().width) : 0,
    childRects: rects.slice(0, 4),
  };
});
console.log("blank-start drag selection:", JSON.stringify(sel));
const selPx = (sel.childRects || []).reduce((a, b) => a + b, 0);
console.log(selPx > 0 ? `PASS: 选区已产生(${(sel.childRects || []).length} 行, 合计 ${selPx}px)` : "FAIL: 无选区");
await page.screenshot({ path: "/tmp/blank-drag.png" });
await b.close();
