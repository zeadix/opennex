import { Actions, type Model, TabNode } from "flexlayout-react";
import { t, type Lang } from "../i18n";

export function panelTitles(lang: Lang): Record<string, string> {
  const T = t(lang);
  return {
    nav: T.navPanel, term: T.cTerminalArea, terminal: T.cTerminalArea,
    ssh: T.ssh, history: T.history, ai: T.ai, settings: T.settings,
    remote: `${T.menuRemote} · ${T.lan}`, "remote-wan": `${T.menuRemote} · ${T.wan}`,
    update: T.updateCheck, favorites: T.favorites, monitor: T.monitor,
    sysmon: T.sysmon, "quick-settings": T.quickSettings, about: T.about, tutorial: T.tutorial,
  };
}

export function translatePanelTitles(model: Model, lang: Lang) {
  const titles = panelTitles(lang);
  model.visitNodes((node) => {
    if (!(node instanceof TabNode)) return;
    const name = titles[node.getComponent() ?? ""];
    if (name && node.getName() !== name) {
      model.doAction(Actions.updateNodeAttributes(node.getId(), { name }));
    }
  });
}
