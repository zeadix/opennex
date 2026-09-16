import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { FiGlobe, FiWifi, FiCopy, FiPlay, FiSquare, FiRefreshCw } from "react-icons/fi";
import { invoke } from "../terminal/tauri";

interface RemoteInfo {
  url: string;
  lanIp: string;
  port: number;
}

interface TunnelStatus {
  state: "idle" | "downloading" | "starting" | "ready" | "failed";
  progress: number;
  url?: string | null;
  error?: string | null;
}

/** In-app QR view for the phone remote control. LAN tab = direct
 * http://lan-ip:port/remote; WAN tab = Cloudflare quick tunnel that
 * exposes the same page to the internet (no account needed). */
export default function RemotePage({ initialTab = "lan" }: { initialTab?: "lan" | "wan" }) {
  const [tab, setTab] = useState<"lan" | "wan">(initialTab);
  const [info, setInfo] = useState<RemoteInfo | null>(null);
  const [lanQr, setLanQr] = useState<string | null>(null);
  const [wanQr, setWanQr] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tunnel, setTunnel] = useState<TunnelStatus | null>(null);

  useEffect(() => {
    invoke<RemoteInfo>("remote_info")
      .then(async (info) => {
        setInfo(info);
        setLanQr(await QRCode.toDataURL(info.url, { margin: 1, width: 220, color: { dark: "#0b0e14", light: "#d6dbe6" } }));
      })
      .catch((e) => setError(String(e)));
  }, []);

  // Poll tunnel status while it is active (download/starting/ready).
  useEffect(() => {
    if (tab !== "wan") return;
    invoke<TunnelStatus>("tunnel_status").then(setTunnel).catch(() => {});
    const id = window.setInterval(() => {
      invoke<TunnelStatus>("tunnel_status")
        .then((s) => {
          setTunnel(s);
          if (s.state === "idle") window.clearInterval(id);
        })
        .catch(() => {});
    }, 1000);
    return () => window.clearInterval(id);
  }, [tab]);

  // QR for the WAN URL once ready.
  useEffect(() => {
    if (tunnel?.state === "ready" && tunnel.url) {
      QRCode.toDataURL(tunnel.url, { margin: 1, width: 220, color: { dark: "#0b0e14", light: "#d6dbe6" } })
        .then(setWanQr)
        .catch(() => {});
    }
  }, [tunnel?.state, tunnel?.url]);

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
          在手机上查看和操作本机终端：局域网直连或广域网隧道。
        </p>

        {/* 页签切换 */}
        <div className="mb-4 inline-flex rounded-lg border border-[var(--border)] bg-[var(--bg)] p-0.5">
          {(["lan", "wan"] as const).map((k) => (
            <button
              key={k}
              className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-[12px] transition-colors ${
                tab === k
                  ? "bg-[var(--accent-dim)] text-[var(--accent)]"
                  : "text-[var(--text-dim)] hover:text-[var(--text)]"
              }`}
              onClick={() => setTab(k)}
            >
              {k === "lan" ? <FiWifi size={13} /> : <FiGlobe size={13} />}
              {k === "lan" ? "局域网" : "广域网"}
            </button>
          ))}
        </div>

        {tab === "lan" && (
          <>
            <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-6">
              {lanQr && (
                <div className="mb-4 flex justify-center">
                  <img src={lanQr} alt="Remote URL QR" className="rounded-lg" width={220} height={220} />
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
          </>
        )}

        {tab === "wan" && (
          <>
            <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-6">
              {tunnel?.state === "ready" && tunnel.url ? (
                <>
                  {wanQr && (
                    <div className="mb-4 flex justify-center">
                      <img src={wanQr} alt="WAN URL QR" className="rounded-lg" width={220} height={220} />
                    </div>
                  )}
                  <div className="mb-2 break-all text-center font-mono text-[12px] text-[var(--accent)]">
                    {tunnel.url}
                  </div>
                  <div className="flex justify-center gap-2">
                    <button
                      className="flex items-center gap-1.5 rounded-md border border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                      onClick={() => navigator.clipboard.writeText(tunnel.url!).catch(() => {})}
                    >
                      <FiCopy size={12} /> 复制地址
                    </button>
                    <button
                      className="flex items-center gap-1.5 rounded-md border border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--danger)] hover:bg-[var(--bg-hover)]"
                      onClick={() => invoke("tunnel_stop").catch(() => {})}
                    >
                      <FiSquare size={12} /> 停止隧道
                    </button>
                  </div>
                </>
              ) : tunnel?.state === "downloading" ? (
                <div className="py-4 text-center">
                  <div className="mb-3 text-[12px] text-[var(--text-dim)]">
                    正在下载 cloudflared（首次使用，约 20-60 MB）…
                  </div>
                  <div className="mx-auto h-1.5 w-64 overflow-hidden rounded-full bg-[var(--bg-active)]">
                    <div
                      className="h-full rounded-full bg-[var(--accent)] transition-all"
                      style={{ width: `${Math.round((tunnel.progress || 0) * 100)}%` }}
                    />
                  </div>
                </div>
              ) : tunnel?.state === "starting" ? (
                <div className="py-6 text-center text-[12px] text-[var(--text-dim)]">
                  正在建立隧道（通常需要 5-20 秒）…
                </div>
              ) : tunnel?.state === "failed" ? (
                <div className="py-4 text-center">
                  <div className="mb-2 text-[12px] text-[var(--danger)]">
                    隧道启动失败{tunnel.error ? `：${tunnel.error}` : ""}
                  </div>
                  <button
                    className="mx-auto flex items-center gap-1.5 rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[11px] text-[var(--accent)]"
                    onClick={() => invoke<TunnelStatus>("tunnel_start").then(setTunnel).catch(() => {})}
                  >
                    <FiRefreshCw size={12} /> 重试
                  </button>
                </div>
              ) : (
                <div className="py-4 text-center">
                  <div className="mb-3 text-[12px] leading-relaxed text-[var(--text-dim)]">
                    通过 Cloudflare 隧道把远程页面暴露到公网，<br />
                    手机在任何网络下均可访问（无需账号，地址临时有效）。
                  </div>
                  <button
                    className="mx-auto flex items-center gap-1.5 rounded-md bg-[var(--accent-dim)] px-4 py-2 text-[12px] font-semibold text-[var(--accent)] hover:brightness-125"
                    onClick={() => invoke<TunnelStatus>("tunnel_start").then(setTunnel).catch(() => {})}
                  >
                    <FiPlay size={12} /> 启动广域网隧道
                  </button>
                </div>
              )}
            </div>
            <div className="mt-4 space-y-1 text-[11px] text-[var(--text-faint)]">
              <div>· 数据经 Cloudflare 边缘 TLS 加密传输，隧道地址停用后立即失效</div>
              <div>· 首次启动会自动下载 cloudflared 到应用数据目录</div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
