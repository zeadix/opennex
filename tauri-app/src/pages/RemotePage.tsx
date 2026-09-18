import { useI18n } from '../i18n-context';
import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { FiGlobe, FiWifi, FiCopy, FiPlay, FiSquare, FiRefreshCw, FiCommand } from "react-icons/fi";
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
  const T = useI18n();
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
        setLanQr(await QRCode.toDataURL(info.url, { margin: 1, width: 220, color: { dark: "#0b0c0f", light: "#ffffff" } }));
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
      QRCode.toDataURL(tunnel.url, { margin: 1, width: 220, color: { dark: "#0b0c0f", light: "#ffffff" } })
        .then(setWanQr)
        .catch(() => {});
    }
  }, [tunnel?.state, tunnel?.url]);

  if (error) {
    return <div className="p-8 text-[13px] text-[var(--danger)]">{error}</div>;
  }
  if (!info) {
    return <div className="p-8 text-[13px] text-[var(--text-faint)]">{T.uLoading}</div>;
  }

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[520px]">
        <h2 className="mb-1 text-[15px] font-semibold">{T.phoneRemote}</h2>
        <p className="mb-4 text-[12px] text-[var(--text-dim)]">
          {T.uRemoteHint}
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
              {k === "lan" ? T.lan : T.wan}
            </button>
          ))}
        </div>

        {tab === "lan" && (
          <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-5">
            {/* 状态头（设计稿）：绿点 + 标题 + 局域网地址 */}
            <div className="mb-4 flex items-center gap-2">
              <span className="h-2 w-2 shrink-0 rounded-full bg-[var(--success)]" />
              <span className="text-[13px] font-semibold">{T.phoneRemote}</span>
              <span className="ml-auto rounded-md border border-[var(--border)] px-2 py-0.5 font-mono text-[10.5px] text-[var(--text-dim)]">
                {info.lanIp}:{info.port}
              </span>
            </div>
            <div className="flex items-start gap-4">
              {lanQr && (
                <img
                  src={lanQr}
                  alt={T.uLanQr}
                  className="w-[128px] shrink-0 rounded-lg border border-[var(--border)] bg-white p-1.5"
                  width={128}
                  height={128}
                />
              )}
              <div className="min-w-0 flex-1">
                <div className="break-all font-mono text-[12.5px] leading-relaxed text-[var(--accent)]">
                  {info.url}
                </div>
                <div className="mt-2 text-[11px] leading-relaxed text-[var(--text-dim)]">
                  {T.phoneRemoteHint}
                </div>
              </div>
            </div>
            {/* 虚拟按键（手机页底部工具栏） */}
            <div className="mt-4 flex items-center gap-1.5 border-t border-[var(--border)] pt-3">
              <FiCommand size={12} className="shrink-0 text-[var(--text-faint)]" />
              {["Ctrl C", "Ctrl V", "Tab", "↑", "Enter"].map((k) => (
                <span
                  key={k}
                  className="rounded border border-[var(--border)] bg-[var(--bg)] px-1.5 py-0.5 font-mono text-[10px] text-[var(--text)]"
                >
                  {k}
                </span>
              ))}
            </div>
            <div className="mt-3 space-y-0.5 text-[10.5px] leading-relaxed text-[var(--text-faint)]">
              <div>{T.remoteNote1}</div>
              <div>{T.remoteNote2}</div>
            </div>
          </div>
        )}

        {tab === "wan" && (
          <>
            <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-6">
              {tunnel?.state === "ready" && tunnel.url ? (
                <>
                  {wanQr && (
                    <div className="mb-4 flex justify-center">
                      <img src={wanQr} alt={T.uWanQr} className="rounded-lg" width={220} height={220} />
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
                      <FiCopy size={12} /> {T.uCopyAddress}
                    </button>
                    <button
                      className="flex items-center gap-1.5 rounded-md border border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--danger)] hover:bg-[var(--bg-hover)]"
                      onClick={() => invoke("tunnel_stop").catch(() => {})}
                    >
                      <FiSquare size={12} /> {T.uStopTunnel}
                    </button>
                  </div>
                </>
              ) : tunnel?.state === "downloading" ? (
                <div className="py-4 text-center">
                  <div className="mb-3 text-[12px] text-[var(--text-dim)]">
                    {T.uTunnelDownloading}
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
                  {T.uTunnelStarting}
                </div>
              ) : tunnel?.state === "failed" ? (
                <div className="py-4 text-center">
                  <div className="mb-2 text-[12px] text-[var(--danger)]">
                    {T.uTunnelFailed}{tunnel.error ? `: ${tunnel.error}` : ""}
                  </div>
                  <button
                    className="mx-auto flex items-center gap-1.5 rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[11px] text-[var(--accent)]"
                    onClick={() => invoke<TunnelStatus>("tunnel_start").then(setTunnel).catch(() => {})}
                  >
                    <FiRefreshCw size={12} /> {T.uRetry}
                  </button>
                </div>
              ) : (
                <div className="py-4 text-center">
                  <div className="mb-3 text-[12px] leading-relaxed text-[var(--text-dim)]">
                    {T.uTunnelHint}
                  </div>
                  <button
                    className="mx-auto flex items-center gap-1.5 rounded-md bg-[var(--accent-dim)] px-4 py-2 text-[12px] font-semibold text-[var(--accent)] hover:brightness-125"
                    onClick={() => invoke<TunnelStatus>("tunnel_start").then(setTunnel).catch(() => {})}
                  >
                    <FiPlay size={12} /> {T.uStartTunnel}
                  </button>
                </div>
              )}
            </div>
            <div className="mt-4 space-y-1 text-[11px] text-[var(--text-faint)]">
              <div>{T.uTunnelSecurity}</div>
              <div>{T.uTunnelFirstRun}</div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
