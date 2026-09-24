// 复现: 纵向分屏 + 大量输出 → 双垂直滚动条 / 水平滚动条闪烁?
import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 900, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s" + Math.random().toString(36).slice(2), wsPort: 19999 });
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

// 新建第二个终端
await page.getByRole("button", { name: "新建终端" }).click();
await page.waitForTimeout(300);
try { await page.getByText("默认 Shell (bash)").click({ timeout: 1500 }); }
catch { await page.getByText("默认 Shell (bash)").click({ force: true, timeout: 1500 }); }
await page.waitForTimeout(2500);

// 把 bash 2 的标签拖到面板底部边缘 → 纵向分屏
const tab = page.locator(".flexlayout__tab_button", { hasText: "bash 2" }).first();
const tb = await tab.boundingBox();
const dock = await page.locator(".term-dock").boundingBox();
if (!tb || !dock) throw new Error("tab/dock not found");
await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height / 2);
await page.mouse.down();
// 分段拖到底部边缘(落在 bottom drop zone)
for (let i = 1; i <= 10; i++) {
  await page.mouse.move(
    dock.x + dock.width / 2,
    tb.y + tb.height / 2 + ((dock.y + dock.height - 20) - (tb.y + tb.height / 2)) * (i / 10),
  );
  await page.waitForTimeout(60);
}
await page.mouse.up();
await page.waitForTimeout(2000);

const sample = () => page.evaluate(() => {
  const doc = document.documentElement;
  const bars = Array.from(document.querySelectorAll(".xterm-viewport")).map((v, i) => {
    const r = v.getBoundingClientRect();
    return { i, w: Math.round(r.width), h: Math.round(r.height), sw: v.scrollWidth, cw: v.clientWidth, sh: v.scrollHeight, ch: v.clientHeight, visible: r.width > 0 };
  });
  const grips = Array.from(document.querySelectorAll("[data-term-slot]")).map((h) => {
    const r = h.getBoundingClientRect();
    const vp = h.querySelector(".xterm-viewport");
    const screen = h.querySelector(".xterm-screen");
    return {
      slot: h.getAttribute("data-term-slot"),
      hostW: Math.round(r.width),
      vpOverX: vp ? vp.scrollWidth - vp.clientWidth : null,
      vpOverY: vp ? vp.scrollHeight - vp.clientHeight : null,
      screenW: screen ? Math.round(screen.getBoundingClientRect().width) : null,
      hostOverflowX: h.scrollWidth - h.clientWidth,
      hostOverflowY: h.scrollHeight - h.clientHeight,
    };
  });
  return { docOverX: doc.scrollWidth - doc.clientWidth, docOverY: doc.scrollHeight - doc.clientHeight, bars, grips };
});
const s1 = await sample();
console.log("sample1:", JSON.stringify(s1));
await page.waitForTimeout(700);
const s2 = await sample();
console.log("sample2:", JSON.stringify(s2));
await page.screenshot({ path: "/tmp/split-scrollbars.png" });
await b.close();
