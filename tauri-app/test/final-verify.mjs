import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 900, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s" + Math.random().toString(36).slice(2), wsPort: 19999 });
      if (cmd === "get_history") return Promise.resolve([{ id: 1, cmd: "docker ps -a", hits: 5 }]);
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
// 建第二个终端并纵向分屏
await page.getByRole("button", { name: "新建终端" }).click();
await page.waitForTimeout(300);
try { await page.getByText("默认 Shell (bash)").click({ timeout: 1500 }); }
catch { await page.getByText("默认 Shell (bash)").click({ force: true, timeout: 1500 }); }
await page.waitForTimeout(2000);
const tab = page.locator(".flexlayout__tab_button", { hasText: "bash 2" }).first();
const tb = await tab.boundingBox();
const dock = await page.locator(".term-dock").boundingBox();
await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height / 2);
await page.mouse.down();
for (let i = 1; i <= 10; i++) {
  await page.mouse.move(dock.x + dock.width / 2, tb.y + tb.height / 2 + ((dock.y + dock.height - 20) - (tb.y + tb.height / 2)) * (i / 10));
  await page.waitForTimeout(60);
}
await page.mouse.up();
await page.waitForTimeout(1500);
// 在 bash 2 输入触发补全
const vis = await page.evaluate(() => {
  const ss = Array.from(document.querySelectorAll(".xterm-screen")).filter((s) => s.getBoundingClientRect().width > 0);
  const s = ss[0].getBoundingClientRect();
  return { x: s.left + 40, y: s.top + 20 };
});
await page.mouse.click(vis.x, vis.y);
await page.keyboard.type("gi");
await page.waitForTimeout(500);
await page.screenshot({ path: "/tmp/final-check.png" });
const out = await page.evaluate(() => {
  const doc = document.documentElement;
  const panel = document.querySelector('div[class*="z-[6000]"]');
  const pr = panel?.getBoundingClientRect();
  const host = panel?.parentElement;
  const hr = host?.getBoundingClientRect();
  // 数一下可视滚动条数量
  const visibleBars = Array.from(document.querySelectorAll("*")).filter((el) => {
    const cs = getComputedStyle(el);
    return (cs.overflowY === "scroll" || cs.overflowX === "scroll") && el.scrollWidth > el.clientWidth + 1;
  }).length;
  return {
    panelOpen: !!panel,
    panelInPane: panel && host ? !!host.querySelector(".xterm") : false,
    panel: pr ? { left: Math.round(pr.left), top: Math.round(pr.top) } : null,
    docOverX: doc.scrollWidth - doc.clientWidth,
    docOverY: doc.scrollHeight - doc.clientHeight,
    scrollScrollContainers: visibleBars,
  };
});
console.log(JSON.stringify(out, null, 2));
await b.close();
