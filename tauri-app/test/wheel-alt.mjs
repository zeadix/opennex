// 验证: alt-screen(TUI)下滚轮 → 方向键上报到达应用;普通缓冲滚轮 → 原生滚动
import { webkit } from "playwright";
import crypto from "node:crypto";
import net from "node:net";
const PORT = 19998;
const received = [];
let sendFn = () => {};
function frame(payload, opcode = 0x2) {
  const len = payload.length;
  const h = len < 126 ? Buffer.from([0x80 | opcode, len])
    : Buffer.concat([Buffer.from([0x80 | opcode, 126]), (() => { const b = Buffer.alloc(2); b.writeUInt16BE(len); return b; })()]);
  return Buffer.concat([h, payload]);
}
const wsserver = net.createServer((socket) => {
  let hs = false, acc = Buffer.alloc(0);
  socket.on("data", (d) => {
    if (!hs) {
      socket.write("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: " +
        crypto.createHash("sha1").update(/Sec-WebSocket-Key: (.+)/.exec(d.toString())[1] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest("base64") + "\r\n\r\n");
      hs = true;
      // 模拟 TUI: 进入 alt-screen + 鼠标上报开 + 画内容
      socket.write(frame(Buffer.from("\x1b[?1049h\x1b[?1000;1002;1006h" + "TUI SCREEN\r\n".repeat(10) + "prompt> ")));
      sendFn = (buf) => socket.write(frame(buf));
      return;
    }
    acc = Buffer.concat([acc, d]);
    if (acc.length < 2) return;
    const masked = !!(acc[1] & 0x80);
    let len = acc[1] & 0x7f, off = 2;
    if (len === 126) { len = acc.readUInt16BE(2); off = 4; }
    const mk = masked ? acc.subarray(off, off + 4) : null;
    if (acc.length < off + (masked ? 4 : 0) + len) return;
    const payload = Buffer.from(acc.subarray(off + (masked ? 4 : 0), off + (masked ? 4 : 0) + len));
    if (masked) for (let i = 0; i < payload.length; i++) payload[i] ^= mk[i % 4];
    acc = acc.subarray(off + (masked ? 4 : 0) + len);
    if (payload.length) {
      received.push(payload.toString("utf8"));
      sendFn = (buf) => socket.write(frame(buf));
    }
  });
});
await new Promise((r) => wsserver.listen(PORT, r));

const b = await webkit.launch({ headless: true });
const ctx = await b.newContext({ viewport: { width: 1100, height: 800 } });
await ctx.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke(cmd) {
      if (cmd === "start_terminal") return Promise.resolve({ session: "s1", wsPort: 19998 });
      if (cmd === "get_history") return Promise.resolve([]);
      return Promise.resolve(null);
    },
    transformCallback(cb) { return typeof cb === "function" ? cb : undefined; },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    plugins: {}, postMessage() {},
  };
  localStorage.setItem("opennex-settings", JSON.stringify({ autoMatch: false }));
});
const page = await ctx.newPage();
await page.goto("http://localhost:5184/test-cursor.html?wsport=19998");
await page.waitForTimeout(9000);
received.length = 0;
await page.mouse.move(550, 400);
await page.mouse.wheel(0, -400);
await page.waitForTimeout(500);
const arrows = received.join("").match(/\x1b\[[AB]/g)?.length ?? 0;
console.log(`alt-screen 滚轮 → 收到方向键上报: ${arrows} 次  ${arrows >= 3 ? "PASS" : "FAIL"}`);
await b.close();
