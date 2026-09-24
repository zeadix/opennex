// WebKit 端到端: 双模式验证
//  跟随开: 自动补全面板 + 历史指令面板都锚定光标(无拖拽条)
//  跟随关: 两个面板均为固定位置(历史=记忆位; 补全=默认右下), 补全面板顶部有拖拽条
import { webkit } from "playwright";

async function run(follow) {
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
    const followParam = new URLSearchParams(location.search).get("follow");
    localStorage.setItem(
      "opennex-settings",
      JSON.stringify({ autoMatch: true, followCursor: followParam === "1" }),
    );
    localStorage.removeItem("opennex-suggest-pos");
    localStorage.removeItem("opennex-palette-pos");
  });
  const page = await ctx.newPage();
  await page.goto(`http://localhost:5183/test-cursor.html?follow=${follow ? 1 : 0}`);
  await page.waitForTimeout(8000);

  // 输入触发自动补全面板
  await page.locator(".xterm-screen").first().click({ position: { x: 60, y: 20 } });
  await page.keyboard.type("gi");
  await page.waitForTimeout(500);
  const suggest = await page.evaluate(() => {
    const hosts = Array.from(document.querySelectorAll("[data-term-slot]")).filter(
      (h) => h.querySelector('div[class*="z-[6000]"]'),
    );
    const host = hosts[0];
    const panel = host?.querySelector('div[class*="z-[6000]"]');
    const r = panel?.getBoundingClientRect();
    return {
      open: !!panel,
      hasGrip: !!panel?.querySelector(".overlay-grip"),
      rect: r ? { left: Math.round(r.left), top: Math.round(r.top) } : null,
    };
  });

  // Alt 唤出历史指令面板
  await page.keyboard.press("Escape");
  await page.waitForTimeout(200);
  await page.keyboard.press("Alt");
  await page.waitForTimeout(1000);
  const palette = await page.evaluate(() => {
    const p = document.querySelector("body > div.animate-fade-up.fixed");
    const r = p?.getBoundingClientRect();
    const d = window.__opennexCursorDebug;
    const e = d ? d.cursorBySlot.get(d.focusedSlot.value) : null;
    const zoom = e?.zoom ?? 1;
    return {
      open: !!p,
      hasGrip: !!p?.querySelector(".overlay-grip"),
      rect: r ? { left: Math.round(r.left), top: Math.round(r.top) } : null,
      caretX: e ? Math.round(e.host.left + e.x * zoom) : null,
      caretBottom: e ? Math.round(e.host.top + (e.y + 1) * (e.h || 20) * zoom) : null,
    };
  });
  await b.close();
  return { suggest, palette };
}

const on = await run(true);
console.log("[跟随开 suggest]", JSON.stringify(on.suggest));
console.log("[跟随开 palette]", JSON.stringify(on.palette));
const off = await run(false);
console.log("[跟随关 suggest]", JSON.stringify(off.suggest));
console.log("[跟随关 palette]", JSON.stringify(off.palette));

// ── 跟随关: 拖拽 + 位置记忆 ──
// 重新起一个 context 模拟"下次调出"
{
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
    // 保留上一步写入的 suggest-pos / palette-pos（模拟"下次调出"）
  });
  const page = await ctx.newPage();
  await page.goto("http://localhost:5183/test-cursor.html");
  await page.waitForTimeout(8000);

  // 自动补全: 拖拽顶部拖拽条 → Esc → 重新输入 → 位置应保持
  await page.locator(".xterm-screen").first().click({ position: { x: 60, y: 20 } });
  await page.keyboard.type("gi");
  await page.waitForTimeout(500);
  const bar = await page.locator('div[class*="z-[6000]"] .overlay-grip').first().boundingBox();
  if (!bar) throw new Error("drag bar missing");
  await page.mouse.move(bar.x + bar.width / 2, bar.y + bar.height / 2);
  await page.mouse.down();
  await page.mouse.move(bar.x + 180, bar.y + 160, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(300);
  const afterDrag = await page.evaluate(() => {
    const p = document.querySelector('div[class*="z-[6000]"]');
    const r = p?.getBoundingClientRect();
    return r ? { left: Math.round(r.left), top: Math.round(r.top) } : null;
  });
  await page.keyboard.press("Escape");
  await page.waitForTimeout(200);
  await page.keyboard.type("gi");
  await page.waitForTimeout(500);
  const afterReopen = await page.evaluate(() => {
    const p = document.querySelector('div[class*="z-[6000]"]');
    const r = p?.getBoundingClientRect();
    return r ? { left: Math.round(r.left), top: Math.round(r.top) } : null;
  });
  console.log("[跟随关 补全拖拽后]", JSON.stringify(afterDrag));
  console.log("[跟随关 补全重开位置]", JSON.stringify(afterReopen));

  // 历史面板: 拖拽顶部条 → Esc → 重开 → 位置保持
  await page.keyboard.press("Escape");
  await page.waitForTimeout(200);
  await page.keyboard.press("Alt");
  await page.waitForTimeout(800);
  const pbar = await page.locator("body > div.animate-fade-up.fixed .overlay-grip").first().boundingBox();
  if (!pbar) throw new Error("palette grip missing");
  await page.evaluate(() => {
    window.__dragLog = [];
    const grip = document.querySelector("body > div.animate-fade-up.fixed .overlay-grip");
    if (!grip) { window.__dragLog.push("no grip el"); return; }
    grip.addEventListener("mousedown", () => window.__dragLog.push("grip mousedown"));
    window.addEventListener("mousemove", () => window.__dragLog.push("move"), { passive: true });
  });
  await page.mouse.move(pbar.x + pbar.width / 2, pbar.y + pbar.height / 2);
  await page.mouse.down();
  await page.mouse.move(pbar.x - 150, pbar.y + 200, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(300);
  const palAfterDrag = await page.evaluate(() => {
    const p = document.querySelector("body > div.animate-fade-up.fixed");
    const r = p?.getBoundingClientRect();
    return {
      rect: r ? { left: Math.round(r.left), top: Math.round(r.top) } : null,
      dragLogHead: (window.__dragLog ?? []).slice(0, 3),
      dragLogLen: (window.__dragLog ?? []).length,
      saved: localStorage.getItem("opennex-palette-pos"),
    };
  });
  await page.keyboard.press("Escape");
  await page.waitForTimeout(200);
  await page.keyboard.press("Alt");
  await page.waitForTimeout(800);
  const palReopen = await page.evaluate(() => {
    const p = document.querySelector("body > div.animate-fade-up.fixed");
    const r = p?.getBoundingClientRect();
    return r ? { left: Math.round(r.left), top: Math.round(r.top) } : null;
  });
  console.log("[跟随关 面板拖拽后]", JSON.stringify(palAfterDrag));
  console.log("[跟随关 面板重开位置]", JSON.stringify(palReopen));
  await b.close();
}
