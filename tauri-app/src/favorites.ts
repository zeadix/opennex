// Favorite-command FOLDERS (egui history_db parity): named folders,
// each holding command items. Persisted in localStorage. The legacy
// flat favorites list (opennex-favorites) is migrated once into a
// default folder.

export interface FavFolder {
  id: string;
  name: string;
  items: string[];
}

const KEY = "opennex-fav-folders-v1";
const LEGACY_KEY = "opennex-favorites";

export function loadFolders(): FavFolder[] {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (Array.isArray(raw) && raw.length > 0) {
      return raw.map((f: any) => ({
        id: String(f.id ?? crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`),
        name: String(f.name ?? "收藏夹"),
        items: Array.isArray(f.items) ? f.items.map(String) : [],
      }));
    }
  } catch {
    /* fall through to legacy migration */
  }
  // One-time migration of the legacy flat favorites list.
  try {
    const legacy = JSON.parse(localStorage.getItem(LEGACY_KEY) ?? "null");
    if (Array.isArray(legacy) && legacy.length > 0) {
      const folders: FavFolder[] = [
        { id: "fav", name: "收藏", items: legacy.map(String) },
      ];
      persistFolders(folders);
      return folders;
    }
  } catch {
    /* ignore */
  }
  return [];
}

export function persistFolders(list: FavFolder[]) {
  localStorage.setItem(KEY, JSON.stringify(list));
}

export function newFolderId(): string {
  return crypto.randomUUID?.() ?? `folder-${Date.now()}-${Math.random()}`;
}
