import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import * as api from "@/lib/api";
import type {
  PluginContribution,
  PluginHostWarn,
} from "@/lib/api/pluginHost";

export type PluginHostEndpoint = {
  baseUrl: string;
  tokens: Record<string, string>;
};

export const PLUGIN_HOST_REFRESH_EVENT = "grok-plugin-host-refresh";

export function requestPluginHostRefresh() {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new Event(PLUGIN_HOST_REFRESH_EVENT));
}

export type PluginContributionsValue = {
  contributions: PluginContribution[];
  warns: PluginHostWarn[];
  endpoint: PluginHostEndpoint | null;
  loading: boolean;
  refresh: () => Promise<void>;
};

const PluginContributionsContext =
  createContext<PluginContributionsValue | null>(null);

export function PluginContributionsProvider({ children }: { children: ReactNode }) {
  const [contributions, setContributions] = useState<PluginContribution[]>([]);
  const [warns, setWarns] = useState<PluginHostWarn[]>([]);
  const [endpoint, setEndpoint] = useState<PluginHostEndpoint | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!api.isTauri()) {
      setContributions([]);
      setWarns([]);
      setEndpoint(null);
      return;
    }
    setLoading(true);
    try {
      const [list, ep] = await Promise.all([
        api.pluginContributionsList(),
        api.pluginUiEndpoint(),
      ]);
      setContributions(Array.isArray(list?.contributions) ? list.contributions : []);
      setWarns(Array.isArray(list?.warns) ? list.warns : []);
      setEndpoint(
        ep?.baseUrl
          ? { baseUrl: ep.baseUrl, tokens: ep.tokens ?? {} }
          : null,
      );
    } catch {
      setContributions([]);
      setWarns([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const onRefresh = () => {
      void refresh();
    };
    window.addEventListener(PLUGIN_HOST_REFRESH_EVENT, onRefresh);
    window.addEventListener("focus", onRefresh);
    return () => {
      window.removeEventListener(PLUGIN_HOST_REFRESH_EVENT, onRefresh);
      window.removeEventListener("focus", onRefresh);
    };
  }, [refresh]);

  const value = useMemo(
    () => ({ contributions, warns, endpoint, loading, refresh }),
    [contributions, warns, endpoint, loading, refresh],
  );

  return (
    <PluginContributionsContext.Provider value={value}>
      {children}
    </PluginContributionsContext.Provider>
  );
}

export function usePluginContributions(): PluginContributionsValue {
  const ctx = useContext(PluginContributionsContext);
  if (!ctx) {
    return {
      contributions: [],
      warns: [],
      endpoint: null,
      loading: false,
      refresh: async () => {},
    };
  }
  return ctx;
}
