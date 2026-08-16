import type { Locale } from "@/i18n";
import type { PluginContribution } from "@/lib/api/pluginHost";

type Props = {
  contributions: PluginContribution[];
  locale: Locale;
  activePlugin?: string | null;
  activePane?: string | null;
  onOpen: (plugin: string, pane: string) => void;
};

function titleFor(
  pane: PluginContribution["sidebar"][number],
  locale: Locale,
): string {
  if (locale === "zh" && pane.titleZh) return pane.titleZh;
  if (locale === "zh-TW" && pane.titleZhTw) return pane.titleZhTw;
  return pane.titleEn;
}

export function PluginNavItems({
  contributions,
  locale,
  activePlugin,
  activePane,
  onOpen,
}: Props) {
  const nav = contributions.flatMap((c) =>
    c.sidebar
      .filter((s) => s.placement !== "more")
      .map((s) => ({ plugin: c.id, pane: s })),
  );
  if (!nav.length) return null;
  return (
    <>
      {nav.map(({ plugin, pane }) => {
        const active = activePlugin === plugin && activePane === pane.id;
        return (
          <button
            key={`${plugin}:${pane.id}`}
            type="button"
            className={"nav-item" + (active ? " nav-item--active" : "")}
            onClick={() => onOpen(plugin, pane.id)}
          >
            <span className="nav-item__icon" aria-hidden>
              ◆
            </span>
            {titleFor(pane, locale)}
          </button>
        );
      })}
    </>
  );
}
