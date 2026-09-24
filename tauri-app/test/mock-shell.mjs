// 测试架：极简 RFC6455 WebSocket 服务器，模拟一个"回显 shell"。
// 连接后下发提示符；客户端发来的 BINARY 帧（终端键入）原样回显，
// TEXT 帧（resize JSON）忽略。让浏览器里的 xterm 拥有真实前进的光标。
import crypto from "node:crypto";
import net from "node:net";

const PORT = 19999;
const PROMPT = Buffer.from("\r\n\x1b[36muser@opennex\x1b[0m:\x1b[34m~\x1b[0m$ ");

function encodeFrame(payload, opcode = 0x2) {
  const len = payload.length;
  let header;
  if (len < 126) {
    header = Buffer.from([0x80 | opcode, len]);
  } else if (len < 65536) {
    header = Buffer.alloc(4);
    header[0] = 0x80 | opcode;
    header[1] = 126;
    header.writeUInt16BE(len, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x80 | opcode;
    header[1] = 127;
    header.writeBigUInt64BE(BigInt(len), 2);
  }
  return Buffer.concat([header, payload]);
}

const server = net.createServer((socket) => {
  let handshake = false;
  let buf = Buffer.alloc(0);
  socket.on("data", (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    if (!handshake) {
      const idx = buf.indexOf("\r\n\r\n");
      if (idx < 0) return;
      const key = /sec-websocket-key: (.+)/i.exec(buf.slice(0, idx).toString())?.[1]?.trim();
      const accept = crypto
        .createHash("sha1")
        .update(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11")
        .digest("base64");
      socket.write(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n" +
          `Sec-WebSocket-Accept: ${accept}\r\n\r\n`,
      );
      handshake = true;
      buf = buf.slice(idx + 4);
      // 模拟 ls -la 的大量输出，用于复现滚动条问题
      let dump = "\r\n";
      for (let i = 0; i < 30; i++) {
        dump += `drwxr-xr-x  61 kunpeng kunpeng  4096 Sep 20 10:21 .dir${i}/  \r\n`;
      }
      socket.write(encodeFrame(Buffer.from(dump)));
      socket.write(encodeFrame(PROMPT));
    }
    while (buf.length >= 2) {
      const opcode = buf[0] & 0x0f;
      let len = buf[1] & 0x7f;
      let off = 2;
      if (len === 126) {
        if (buf.length < 4) return;
        len = buf.readUInt16BE(2);
        off = 4;
      } else if (len === 127) {
        if (buf.length < 10) return;
        len = Number(buf.readBigUInt64BE(2));
        off = 10;
      }
      const masked = (buf[1] & 0x80) !== 0;
      if (buf.length < off + (masked ? 4 : 0) + len) return;
      let payload = buf.slice(off + (masked ? 4 : 0), off + (masked ? 4 : 0) + len);
      if (masked) {
        const mask = buf.slice(off, off + 4);
        payload = Buffer.from(payload.map((b, i) => b ^ mask[i % 4]));
      }
      buf = buf.slice(off + (masked ? 4 : 0) + len);
      if (opcode === 0x8) {
        socket.write(encodeFrame(Buffer.alloc(0), 0x8));
        socket.end();
        return;
      }
      if (opcode === 0x2) socket.write(encodeFrame(payload)); // 键入回显
      // opcode 0x1（resize JSON）与 0x9（ping）忽略/按需 pong
      if (opcode === 0x9) socket.write(encodeFrame(payload, 0xa));
    }
  });
  socket.on("error", () => {});
});

server.listen(PORT, "127.0.0.1", () => console.log(`mock shell ws on :${PORT}`));
