import { webkit } from "playwright";
const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke() { return Promise.resolve(null); },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: true, followCursor: true }));
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(2500);
// 打开快捷设置的下拉(聚焦态截图用 hover) + 整体截图
await page.locator("select").first?.();
  // 视图菜单 → 快捷设置
  await page.getByRole("button", { name: "视图" }).click();
  await page.waitForTimeout(300);
  await page.getByText("快捷设置", { exact: true }).first().click();
  await page.waitForTimeout(500);
await page.screenshot({ path: "/tmp/qs-light.png" });
// 切深色主题对比
await page.evaluate(() => {
  const themes = JSON.parse(localStorage.getItem("opennex-user-themes") ?? "[]");
  localStorage.setItem("opennex-theme", "paper");
});
await page.reload();
await page.waitForTimeout(2500);
await page.screenshot({ path: "/tmp/qs-dark.png" });
await b.close();
