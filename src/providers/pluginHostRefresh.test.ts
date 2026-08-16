import { describe, expect, it, vi } from "vitest";
import {
  PLUGIN_HOST_REFRESH_EVENT,
  requestPluginHostRefresh,
} from "./PluginContributionsProvider";

describe("requestPluginHostRefresh", () => {
  it("dispatches the host refresh event so contributions rescan after install", () => {
    const seen: string[] = [];
    const fake = {
      dispatchEvent(e: Event) {
        seen.push(e.type);
        return true;
      },
    };
    vi.stubGlobal("window", fake);
    requestPluginHostRefresh();
    vi.unstubAllGlobals();
    expect(seen).toEqual([PLUGIN_HOST_REFRESH_EVENT]);
  });
});
