// 双终端同时可见：各自输入，面板必须各随各的光标。
import { webkit } from "playwright";

const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s" + Math.random(), wsPort: 19999 });
      if (cmd === "get_history")
        return Promise.resolve([
          { id: 1, cmd: "docker ps -a", hits: 5 },
          { id: 2, cmd: "git status", hits: 3 },
          { id: 3, cmd: "kubectl get pods", hits: 2 },
        ]);
      if (cmd === "list_shells") return Promise.resolve(["bash"]);
      if (cmd === "list_path_commands")
        return Promise.resolve(["docker", "docker-compose", "kubectl", "ls", "git", "grep", "curl"]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {},
    postMessage() {},
  };
  localStorage.setItem(
    "opennex-settings",
    JSON.stringify({ autoMatch: true, followCursor: true, suggestSource: "auto" }),
  );
  localStorage.removeItem("opennex-suggest-pos");
  localStorage.removeItem("opennex-palette-pos");
});
const page = await ctx.newPage();
await page.goto("http://localhost:5183/test-cursor.html");
await page.waitForTimeout(8000);

// 新建第二个终端
await page.getByRole("button", { name: "新建终端" }).click();
await page.waitForTimeout(300);
try {
  await page.getByText("默认 Shell (bash)").click({ timeout: 1500 });
} catch {
  await page.getByText("默认 Shell (bash)").click({ force: true, timeout: 1500 });
}
await page.waitForTimeout(2000);

async function measure() {
  return page.evaluate(() => {
    const panels = Array.from(document.querySelectorAll('div[class*="z-[6000]"]'));
    const terms = Array.from(document.querySelectorAll(".xterm")).map((x) => {
      const host = x.parentElement;
      const r = host.getBoundingClientRect();
      return { left: Math.round(r.left), top: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height), visible: r.width > 0 };
    });
    const panelsGeo = panels.map((p) => {
      const r = p.getBoundingClientRect();
      const host = p.parentElement.getBoundingClientRect();
      return {
        left: Math.round(r.left), top: Math.round(r.top), right: Math.round(r.right),
        paneLeft: Math.round(host.left), paneTop: Math.round(host.top),
        paneW: Math.round(host.width), paneH: Math.round(host.height),
        inside: r.left >= host.left - 1 && r.right <= host.right + 1 && r.top >= host.top - 1 && r.bottom <= host.bottom + 1,
        head: p.textContent.slice(0, 20),
      };
    });
    return { terms, panels: panelsGeo };
  });
}

async function clickVisibleTerminal() {
  const r = await page.evaluate(() => {
    const ss = Array.from(document.querySelectorAll(".xterm-screen")).filter(
      (s) => s.getBoundingClientRect().width > 0,
    );
    const s = ss[0]?.getBoundingClientRect();
    return s ? { x: s.left + 60, y: s.top + 20 } : null;
  });
  if (!r) throw new Error("no visible terminal");
  await page.mouse.click(r.x, r.y);
}

// 终端2 输入 ku（当前选中）
await clickVisibleTerminal();
await page.keyboard.type("ku");
await page.waitForTimeout(500);
const m2 = await measure();
console.log("[bash 2 输入 ku]", JSON.stringify(m2));

// 切到 bash 1 输入 gi（面板必须跟到 bash 1 的光标）
await page.getByRole("tab", { name: "bash 1" }).click();
await page.waitForTimeout(400);
await clickVisibleTerminal();
await page.keyboard.type("gi");
await page.waitForTimeout(500);
const m1 = await measure();
console.log("[bash 1 输入 gi]", JSON.stringify(m1));

await b.close();
