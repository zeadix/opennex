import { useEffect, useState } from "react";
import { applyTheme, loadThemeId } from "./themes";

export function useTheme(): [string, (id: string) => void] {
  const [id, setId] = useState(loadThemeId());
  useEffect(() => {
    applyTheme(id);
  }, [id]);
  const set = (next: string) => setId(next);
  return [id, set];
}
