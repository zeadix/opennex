import { createContext, useContext, useMemo, type ReactNode } from "react";
import { t, type Lang } from "./i18n";

const LanguageContext = createContext<Lang>("zh");

export function I18nProvider({ lang, children }: { lang: Lang; children: ReactNode }) {
  return <LanguageContext.Provider value={lang}>{children}</LanguageContext.Provider>;
}

export function useI18n() {
  const lang = useContext(LanguageContext);
  return useMemo(() => t(lang), [lang]);
}
