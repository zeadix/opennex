// 实测: UI zoom(CSS zoom)下 xterm 框选是否错位。
// 方法: 分别以 uiFontSize=13(zoom 1)与 17(zoom≈1.31)启动,
// 用视觉坐标(按 .xterm-screen 的 rect 与列数换算)从 cell(3,2) 拖到
// cell(17,4),读取 xterm 的 selection 层位置与几何,对比换算误差。
import { webkit } from "playwright";

async function probe(uiFontSize) {
  const b = await webkit.launch({ headless: true });
  const ctx = await b.newContext({ viewport: { width: 1100, height: 800 } });
  await ctx.addInitScript(() => {
    window.__TAURI_INTERNALS__ = {
      invoke(cmd) {
        if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19999 });
        if (cmd === "get_history") return Promise.resolve([]);
        return Promise.resolve(null);
      },
      transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
      plugins: {}, postMessage() {},
    };
  });
  const page = await ctx.newPage();
  await page.goto(`http://localhost:5183/test-cursor.html?uifs=${uiFontSize}`);
  await page.waitForTimeout(9000);
  // 鼠标拖选(视觉坐标):从 cell(3,2) 拖到 cell(17,4)
  const drag = await page.evaluate(() => {
    const screen = document.querySelector(".xterm-screen");
    const rows = document.querySelector(".xterm-rows");
    if (!screen || !rows) return null;
    const sr = screen.getBoundingClientRect();
    const lh = parseFloat(getComputedStyle(rows.firstElementChild || rows).lineHeight) || 20;
    const cw = sr.width / Math.max(1, Math.round(sr.width / 9));
    const pt = (col, row) => ({ x: sr.left + col * cw + cw / 2, y: sr.top + row * lh + lh / 2 });
    return { from: pt(3, 2), to: pt(17, 4), cw, lh };
  });
  if (drag) {
    await page.mouse.move(drag.from.x, drag.from.y);
    await page.mouse.down();
    await page.mouse.move(drag.to.x, drag.to.y, { steps: 8 });
    await page.mouse.up();
    await page.waitForTimeout(300);
  }
  const sel = await page.evaluate(() => {
    const el = document.querySelector(".xterm-selection");
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { l: Math.round(r.left), t: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) };
  });
  const out = await page.evaluate(() => {
    const screen = document.querySelector(".xterm-screen");
    const inter = document.querySelector(".xterm-helpers")?.parentElement; // viewport/screen 容器
    const host = document.querySelector("[data-term-slot]");
    if (!screen || !host) return { err: "no terminal" };
    const sr = screen.getBoundingClientRect();
    const cols = Number(screen.getAttribute("style")?.match(/width: ([\d.]+)px/)?.[1] ?? 0);
    // cell 尺寸: 用 xterm 的 style helper 精确取
    const rows = host.querySelectorAll(".xterm-rows > div").length;
    return {
      zoom: getComputedStyle(host.querySelector(".xterm")).zoom || getComputedStyle(document.body).zoom || "n/a",
      screenRect: { l: Math.round(sr.left), t: Math.round(sr.top), w: Math.round(sr.width), h: Math.round(sr.height) },
      hostRect: (() => { const r = host.getBoundingClientRect(); return { l: Math.round(r.left), t: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) }; })(),
      offsetW: screen.offsetWidth,
      cssW: sr.width,
      selection: null,
    };
  });
  out.selection = sel;
  // 校准度:视觉宽与布局宽的比值(1 = 坐标系一致,框选必然对齐)
  out.coordSkew = Math.round((out.cssW / out.offsetW) * 1000) / 1000;
  await b.close();
  return out;
}

for (const fs of [13, 17]) {
  console.log(`uiFontSize=${fs}:`, JSON.stringify(await probe(fs)));
}
