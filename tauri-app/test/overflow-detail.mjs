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
// 先创建第二个终端并纵向分屏
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
  await page.mouse.move(
    dock.x + dock.width / 2,
    tb.y + tb.height / 2 + ((dock.y + dock.height - 20) - (tb.y + tb.height / 2)) * (i / 10),
  );
  await page.waitForTimeout(60);
}
await page.mouse.up();
await page.waitForTimeout(2000);
const out = await page.evaluate(() => {
  const host = document.querySelector("[data-term-slot]");
  if (!host) return { err: "no host" };
  const kids = Array.from(host.children).map((c) => {
    const r = c.getBoundingClientRect();
    const hrs = host.getBoundingClientRect();
    return {
      tag: c.tagName,
      cls: String(c.className).slice(0, 50),
      offsetH: c.offsetHeight,
      topInHost: Math.round(r.top - hrs.top),
      bottomInHost: Math.round(r.bottom - hrs.top),
    };
  });
  const term = host.querySelector(".xterm");
  const rows = host.querySelector(".xterm-rows");
  const deep = Array.from(host.querySelectorAll("*"))
    .map((c) => {
      const r = c.getBoundingClientRect();
      const hrs = host.getBoundingClientRect();
      return { cls: String(c.className).slice(0, 44), bottomInHost: Math.round(r.bottom - hrs.top), h: Math.round(r.height) };
    })
    .filter((x) => x.bottomInHost > 350)
    .sort((a, b) => b.bottomInHost - a.bottomInHost)
    .slice(0, 6);
  return {
    hostClientH: host.clientHeight,
    hostScrollH: host.scrollHeight,
    overflow: host.scrollHeight - host.clientHeight,
    kids,
    deep,
    termH: term?.offsetHeight ?? null,
    termRows: rows?.childElementCount ?? null,
    screenH: host.querySelector(".xterm-screen")?.offsetHeight ?? null,
  };
});
console.log(JSON.stringify(out, null, 2));
await b.close();
