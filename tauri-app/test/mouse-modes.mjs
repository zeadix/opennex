// 实验三连:
//  A. 普通缓冲滚轮 → xterm viewport 应原生滚动
//  B. 开启 SGR 鼠标上报(?1000;1002;1006h)后点击 → 应收到 \x1b[<b;x;yM
//  C. 鼠标模式下滚轮 → 应收到 \x1b[<64;x;yM
import { webkit } from "playwright";
import crypto from "node:crypto";
import net from "node:net";

const PORT = 19998;
const received = [];   // 应用侧收到的所有字节(utf8)
const PROMPT = Buffer.from("\r\n\u001b[36muser@opennex\u001b[0m:\u001b[34m~\u001b[0m$ ");
function frame(payload, opcode = 0x2) {
  const len = payload.length;
  const h = len < 126 ? Buffer.from([0x80 | opcode, len])
    : Buffer.concat([Buffer.from([0x80 | opcode, 126]), (() => { const b = Buffer.alloc(2); b.writeUInt16BE(len); return b; })()]);
  return Buffer.concat([h, payload]);
}
const wsserver = net.createServer((socket) => {
  console.log("[ws] client connected");
  let hs = false, acc = Buffer.alloc(0);
  socket.on("data", (d) => {
    if (!hs) {
      socket.write("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: " +
        crypto.createHash("sha1").update(/Sec-WebSocket-Key: (.+)/.exec(d.toString())[1] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest("base64") + "\r\n\r\n");
      hs = true;
      console.log("[ws] handshake done");
      const lines = [];
      for (let i = 0; i < 80; i++) lines.push(`filler-${i} .............................`);
      socket.write(frame(Buffer.from(lines.join("\r\n") + "\r\n" + PROMPT.toString("utf8"))));
      return;
    }
    acc = Buffer.concat([acc, d]);
    // 完整 WS 帧解析:客户端帧必须解掩码(XOR mask-key)
    if (acc.length < 2) return;
    const masked = !!(acc[1] & 0x80);
    let len = acc[1] & 0x7f;
    let off = 2;
    if (len === 126) { if (acc.length < 4) return; len = acc.readUInt16BE(2); off = 4; }
    else if (len === 127) { if (acc.length < 10) return; len = Number(acc.readBigUInt64BE(2)); off = 10; }
    const maskLen = masked ? 4 : 0;
    if (acc.length < off + maskLen + len) return;
    let payload = Buffer.from(acc.subarray(off + maskLen, off + maskLen + len));
    if (masked) {
      const mk = acc.subarray(off, off + 4);
      for (let i = 0; i < payload.length; i++) payload[i] ^= mk[i % 4];
    }
    acc = acc.subarray(off + maskLen + len);
    if (payload.length) {
      received.push(payload.toString("utf8"));
      socket.write(frame(payload)); // echo 键入
    }
    // 帧缓冲里可能还有后续帧——极简处理:清空(实验足够)
  });
  globalThis.sendToTerm = (buf) => socket.write(frame(buf));
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
page.on("console", (m) => console.log("[page]", m.text().slice(0, 120)));
await page.goto("http://localhost:5184/test-cursor.html?wsport=19998&uifs=13");
await page.waitForTimeout(9000);

await page.screenshot({ path: "/tmp/mouse-debug.png" });
// A0. 内部结构取证
const probe = await page.evaluate(() => {
  const q = (s2) => document.querySelector(s2);
  const r = (s2) => { const e = q(s2); return e ? Math.round(e.getBoundingClientRect().height) : null; };
  return {
    rowsChildren: q(".xterm-rows")?.children.length,
    rowsH: r(".xterm-rows"),
    scrollAreaH: r(".xterm-scroll-area"),
    vpH: r(".xterm-viewport"),
    vpScrollH: q(".xterm-viewport")?.scrollHeight,
    screenH: r(".xterm-screen"),
    hostH: r("[data-term-slot]"),
    lastRowText: q(".xterm-rows")?.lastElementChild?.textContent?.slice(0, 40),
  };
});
console.log("结构:", JSON.stringify(probe, null, 1));

// A. 普通缓冲滚轮
const before = await page.evaluate(() => document.querySelector(".xterm-viewport").scrollTop);
await page.mouse.move(550, 400);
await page.mouse.wheel(0, -600);
await page.waitForTimeout(400);
const afterA = await page.evaluate(() => document.querySelector(".xterm-viewport").scrollTop);
console.log(`A. 普通滚轮: scrollTop ${before} -> ${afterA}  ${afterA < before ? "PASS(向上滚动了)" : "FAIL"}`);

// B. SGR 鼠标上报:开模式,点击 cell(12,5)
globalThis.sendToTerm(Buffer.from("\x1b[?1000;1002;1006h"));
await page.waitForTimeout(300);
received.length = 0;
await page.mouse.click(550, 400);
await page.waitForTimeout(400);
const gotB = received.join("");
console.log(`B. 点击上报: ${JSON.stringify(gotB)}  ${gotB.includes("\x1b[<") ? "PASS" : "FAIL"}`);

// C. 鼠标模式滚轮
received.length = 0;
await page.mouse.wheel(0, -240);
await page.waitForTimeout(400);
const gotC = received.join("");
console.log(`C. 滚轮上报: ${JSON.stringify(gotC)}  ${gotC.includes("\x1b[<64") ? "PASS" : "FAIL"}`);

await b.close();
