import { useI18n } from '../i18n-context';
import { useState } from "react";
import { FiCopy, FiEdit2, FiPlus, FiServer, FiTrash2 } from "react-icons/fi";

export interface SshHost {
  id: number;
  name: string;
  user: string;
  host: string;
  port: number;
}

const KEY = "opennex-ssh-hosts";

export function loadHosts(): SshHost[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "[]");
  } catch {
    return [];
  }
}
export function saveHosts(hosts: SshHost[]) {
  localStorage.setItem(KEY, JSON.stringify(hosts));
}

export default function SshPage({
  hosts,
  onHosts,
  onConnect,
}: {
  hosts: SshHost[];
  onHosts: (h: SshHost[]) => void;
  onConnect: (host: SshHost) => void;
}) {
  const T = useI18n();
  const [adding, setAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState({ name: "", user: "", host: "", port: 22 });

  const openForm = (h?: SshHost) => {
    setEditingId(h?.id ?? null);
    setForm(h ? { name: h.name, user: h.user, host: h.host, port: h.port } : { name: "", user: "", host: "", port: 22 });
    setAdding(true);
  };

  const submit = () => {
    if (!form.host || !form.user) return;
    if (editingId != null) {
      onHosts(
        hosts.map((h) =>
          h.id === editingId
            ? { ...h, name: form.name || `${form.user}@${form.host}`, user: form.user, host: form.host, port: form.port || 22 }
            : h,
        ),
      );
    } else {
      const h: SshHost = {
        id: Date.now(),
        name: form.name || `${form.user}@${form.host}`,
        user: form.user,
        host: form.host,
        port: form.port || 22,
      };
      onHosts([...hosts, h]);
    }
    setForm({ name: "", user: "", host: "", port: 22 });
    setEditingId(null);
    setAdding(false);
  };

  return (
    <div className="h-full min-w-0 overflow-y-auto p-3">
      <div className="w-full min-w-0">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-[15px] font-semibold">{T.sshHosts}</h2>
          <button
            onClick={() => (adding ? (setAdding(false), setEditingId(null)) : openForm())}
            className="flex items-center gap-1.5 rounded-md border border-[var(--border)] px-2.5 py-1.5 text-[12px] hover:border-[var(--accent)] hover:text-[var(--accent)]"
          >
            <FiPlus size={13} /> {T.uNew}
          </button>
        </div>

        {adding && (
          <div className="mb-4 space-y-2 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <div className="grid grid-cols-2 gap-2">
              <input className="dialog-input" placeholder={T.name} value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })} />
              <input className="dialog-input" placeholder={T.port} value={form.port}
                onChange={(e) => setForm({ ...form, port: Number(e.target.value) || 22 })} />
              <input className="dialog-input col-span-2" placeholder={T.hostAddr} value={form.host}
                onChange={(e) => setForm({ ...form, host: e.target.value })} />
              <input className="dialog-input col-span-2" placeholder={T.user} value={form.user}
                onChange={(e) => setForm({ ...form, user: e.target.value })} />
            </div>
            <div className="flex justify-end gap-2 pt-1">
              <button className="rounded-md px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]"
                onClick={() => setAdding(false)}>{T.cancel}</button>
              <button className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
                onClick={submit}>{editingId != null ? T.uSaveChanges : T.save}</button>
            </div>
          </div>
        )}

        <div className="space-y-1.5">
          {hosts.length === 0 && (
            <div className="rounded-lg border border-dashed border-[var(--border)] py-10 text-center text-[12px] text-[var(--text-faint)]">
              {T.noHosts}
            </div>
          )}
          {hosts.map((h) => (
            <div key={h.id}
              className="group flex cursor-pointer items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] px-4 py-3 transition-colors hover:border-[var(--accent)]"
              onClick={() => onConnect(h)}>
              <FiServer size={16} className="text-[var(--text-faint)]" />
              <div className="min-w-0 flex-1">
                <div className="text-[13px]">{h.name}</div>
                <div className="font-mono text-[11px] text-[var(--text-faint)]">
                  {h.user}@{h.host}:{h.port}
                </div>
              </div>
              <button className="icon-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--accent)]"
                title={T.uEdit}
                onClick={(e) => {
                  e.stopPropagation();
                  openForm(h);
                }}>
                <FiEdit2 size={14} />
              </button>
              <button className="icon-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--accent)]"
                title={T.uDuplicateHost}
                onClick={(e) => {
                  e.stopPropagation();
                  const copy: SshHost = { ...h, id: Date.now(), name: T.uHostCopyName.replace("{name}", () => h.name) };
                  onHosts([...hosts, copy]);
                }}>
                <FiCopy size={14} />
              </button>
              <button className="icon-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"
                title={T.uDelete}
                onClick={(e) => {
                  e.stopPropagation();
                  onHosts(hosts.filter((x) => x.id !== h.id));
                }}>
                <FiTrash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
