import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { invoke } from "../terminal/tauri";

interface RemoteInfo {
  url: string;
  lanIp: string;
  port: number;
}

/** In-app QR view for the phone remote control (phone opens
 * http://lan-ip:port/remote and gets a live terminal view + input). */
export default function RemotePage() {
  const [info, setInfo] = useState<RemoteInfo | null>(null);
  const [qr, setQr] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<RemoteInfo>("remote_info")
      .then(async (info) => {
        setInfo(info);
        setQr(await QRCode.toDataURL(info.url, { margin: 1, width: 220, color: { dark: "#0b0e14", light: "#d6dbe6" } }));
      })
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return <div className="p-8 text-[13px] text-[var(--danger)]">{error}</div>;
  }
  if (!info) {
    return <div className="p-8 text-[13px] text-[var(--text-faint)]">加载中…</div>;
  }

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[520px]">
        <h2 className="mb-1 text-[15px] font-semibold">手机远程控制</h2>
        <p className="mb-4 text-[12px] text-[var(--text-dim)]">
          手机与电脑处于同一局域网时，扫码或访问下方地址即可在手机上查看和操作终端。
        </p>
        <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-6">
          {qr && (
            <div className="mb-4 flex justify-center">
              <img src={qr} alt="Remote URL QR" className="rounded-lg" width={220} height={220} />
            </div>
          )}
          <div className="mb-2 break-all text-center font-mono text-[12px] text-[var(--accent)]">
            {info.url}
          </div>
          <div className="text-center text-[11px] text-[var(--text-faint)]">
            局域网 IP {info.lanIp} · 端口 {info.port} · 同一 Wi-Fi 下可用
          </div>
        </div>
        <div className="mt-4 space-y-1 text-[11px] text-[var(--text-faint)]">
          <div>· 远程页面支持查看终端输出、发送命令、切换会话</div>
          <div>· 会话结束或应用退出后远程访问自动失效</div>
        </div>
      </div>
    </div>
  );
}
