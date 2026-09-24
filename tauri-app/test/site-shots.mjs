// 官网截图: 用 WebKit 跑真实前端 + mock shell/AI, 产出各功能高清截图
import { webkit } from "playwright";
import fs from "node:fs";

const OUT = "/home/kunpengwang/proj/my/opennex/website/public/ss";
fs.mkdirSync(OUT, { recursive: true });

const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
});
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd, args) {
      if (cmd === "start_terminal")
        return Promise.resolve({ session: "s" + Math.random().toString(36).slice(2), wsPort: 19999 });
      if (cmd === "ai_chat")
        return Promise.resolve(
          "可以按这个顺序排查：\n1. df -h 总览各分区占用\n2. du -sh /* 2>/dev/null | sort -h 找出大目录\n3. journalctl --disk-usage 查看日志占用\n把结果发给我，我帮你继续定位。",
        );
      if (cmd === "get_history")
        return Promise.resolve([
          { id: 1, cmd: "docker compose up -d", hits: 6 },
          { id: 2, cmd: "git status", hits: 4 },
          { id: 3, cmd: "kubectl get pods -A", hits: 3 },
          { id: 4, cmd: "cargo build --release", hits: 2 },
          { id: 5, cmd: "ssh deploy@203.0.113.7", hits: 2 },
        ]);
      if (cmd === "list_path_commands")
        return Promise.resolve(["docker", "docker-compose", "kubectl", "ls", "git", "grep", "curl"]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-lang", "zh"); // 界面语言固定简体中文
  localStorage.setItem(
    "opennex-settings",
    JSON.stringify({ autoMatch: true, followCursor: true, suggestSource: "auto" }),
  );
  localStorage.removeItem("opennex-suggest-pos");
  localStorage.removeItem("opennex-palette-pos");
  localStorage.removeItem("opennex-palette-pin-pos");
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(9000); // 布局等待 + shell 启动 + 模拟 ls 输出

// 打开 AI 助手 + 快捷设置 面板
await page.getByRole("button", { name: "视图" }).click();
await page.waitForTimeout(300);
await page.getByText("AI助手", { exact: true }).first().click();
await page.waitForTimeout(600);
await page.getByRole("button", { name: "视图" }).click();
await page.waitForTimeout(300);
await page.getByText("快捷设置", { exact: true }).first().click();
await page.waitForTimeout(600);

// AI 面板提问 → 罐头回复
await page.locator('input[placeholder*="问点"]').fill("磁盘占用太高怎么排查？");
await page.keyboard.press("Enter");
await page.waitForTimeout(900);

// 终端聚焦 + 输入触发补全面板（跟随光标）
const vis = await page.evaluate(() => {
  const ss = Array.from(document.querySelectorAll(".xterm-screen")).filter(
    (s) => s.getBoundingClientRect().width > 0,
  );
  const s = ss[0].getBoundingClientRect();
  return { x: s.left + 80, y: s.top + 24 };
});
await page.mouse.click(vis.x, vis.y);
await page.keyboard.type("gi");
await page.waitForTimeout(600);

await page.screenshot({ path: `${OUT}/home-hero.png` });
console.log("home-hero done");

// ── AI 面板特写 ──
const aiRect = await page.evaluate(() => {
  const inputs = Array.from(document.querySelectorAll('input[placeholder*="问点"]'));
  const host = inputs[0]?.closest('[class*="tabpanel"], .flexlayout__tab_content, div');
  let el = inputs[0];
  for (let i = 0; i < 8 && el; i++) {
    el = el.parentElement;
    const r = el.getBoundingClientRect();
    if (r.width > 260 && r.height > 380) return { left: r.left, top: r.top, w: r.width, h: r.height };
  }
  return null;
});
if (aiRect) {
  await page.screenshot({
    path: `${OUT}/ai-assistant.png`,
    clip: { x: aiRect.left, y: aiRect.top, width: aiRect.w, height: aiRect.h },
  });
  console.log("ai-assistant done");
}

// ── 补全跟随光标特写（终端区 + 补全面板）──
const termRect = await page.evaluate(() => {
  const ss = Array.from(document.querySelectorAll(".xterm-screen")).filter(
    (s) => s.getBoundingClientRect().width > 0,
  );
  const r = ss[0].getBoundingClientRect();
  const panel = document.querySelector('div[class*="z-[6000]"]')?.getBoundingClientRect();
  const left = Math.min(r.left, panel?.left ?? r.left) - 14;
  const top = Math.min(r.top, panel?.top ?? r.top) - 40;
  const right = Math.max(r.right, panel?.right ?? r.right) + 14;
  const bottom = Math.max(r.bottom, panel?.bottom ?? r.bottom) + 14;
  return {
    x: Math.max(0, left), y: Math.max(0, top),
    width: Math.min(right, 1440) - Math.max(0, left),
    height: Math.min(bottom, 900) - Math.max(0, top),
  };
});
await page.screenshot({ path: `${OUT}/completion-follow.png`, clip: termRect });
console.log("completion-follow done");

// ── 指令面板：Esc 关补全 → Alt 唤出历史面板 ──
await page.keyboard.press("Escape");
await page.waitForTimeout(200);
await page.keyboard.press("Alt");
await page.waitForTimeout(800);
const palRect = await page.evaluate(() => {
  const p = document.querySelector("body > div.animate-fade-up.fixed");
  const r = p?.getBoundingClientRect();
  return r
    ? {
        x: Math.max(0, r.left - 16), y: Math.max(0, r.top - 16),
        width: Math.min(r.width + 32, 1440 - Math.max(0, r.left - 16)),
        height: Math.min(r.height + 32, 900 - Math.max(0, r.top - 16)),
      }
    : null;
});
if (palRect) {
  await page.screenshot({ path: `${OUT}/command-palette.png`, clip: palRect });
  console.log("command-palette done");
}

// ── 快捷设置面板特写（补全指令来源下拉）──
await page.keyboard.press("Escape");
await page.waitForTimeout(200);
const qsRect = await page.evaluate(() => {
  const heads = Array.from(document.querySelectorAll("h2")).find((h) => h.textContent === "快捷设置");
  const panel = heads?.closest('[class*="tabpanel"], .flexlayout__tab_content, div');
  const r = (panel ?? heads).getBoundingClientRect();
  return { x: r.left, y: r.top, width: r.width, height: Math.min(r.height, 320) };
});
if (qsRect.width > 80) {
  await page.screenshot({ path: `${OUT}/quick-settings.png`, clip: qsRect });
  console.log("quick-settings done");
}
await b.close();
console.log("ALL DONE");
