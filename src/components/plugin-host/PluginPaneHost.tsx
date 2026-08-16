import { useCallback, useEffect, useMemo, useRef } from "react";
import { createT, type Locale } from "@/i18n";
import type { PluginContribution } from "@/lib/api/pluginHost";
import {
  dispatchHostRequest,
  parseHostReq,
  type BridgeContext,
} from "@/lib/pluginHost/bridge";
import {
  composeNewSession,
  finishJob,
  runNewSession,
  type PluginJob,
} from "@/lib/pluginHost/jobs";
import { storageForPlugin } from "@/lib/pluginHost/storage";
import {
  hostReadyPayload,
  sessionDonePayload,
  sessionNeedsUserPayload,
  sessionStartedPayload,
} from "@/lib/pluginHost/hostEvents";
import * as api from "@/lib/api";
import type { SessionSnapshot } from "@/lib/session";
import { saveComposerSessionDraft } from "@/lib/composerSessionDraft";
import { GlassModal } from "@/components/GlassModal";
import { useState } from "react";

type DialogState =
  | null
  | {
      kind: "notice" | "confirm" | "prompt";
      title: string;
      body: string;
      resolve: (v: boolean | string | null) => void;
    };

type Props = {
  contribution: PluginContribution;
  paneId: string;
  baseUrl: string;
  token: string;
  locale: Locale;
  theme: string;
  tokens: Record<string, string>;
  hidden?: boolean;
};

function iframeSrc(
  baseUrl: string,
  pluginId: string,
  token: string,
  entry: string,
): string {
  const rel = entry.startsWith("ui/") ? entry.slice(3) : entry;
  return `${baseUrl.replace(/\/$/, "")}/plugin-ui/${pluginId}/${token}/${rel}`;
}

export function PluginPaneHost({
  contribution,
  paneId,
  baseUrl,
  token,
  locale,
  theme,
  tokens,
  hidden,
}: Props) {
  const tr = createT(locale);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const jobsRef = useRef<Map<string, PluginJob>>(new Map());
  const [dialog, setDialog] = useState<DialogState>(null);
  const pane = contribution.sidebar.find((s) => s.id === paneId);
  const src = pane
    ? iframeSrc(baseUrl, contribution.id, token, pane.entry)
    : "";

  const origin = useMemo(() => {
    try {
      return new URL(baseUrl).origin;
    } catch {
      return "";
    }
  }, [baseUrl]);

  const postToFrame = useCallback(
    (data: unknown) => {
      const win = iframeRef.current?.contentWindow;
      if (!win || !origin) return;
      win.postMessage(data, origin);
    },
    [origin],
  );

  const openDialog = useCallback(
    (kind: "notice" | "confirm" | "prompt", title: string, body: string) =>
      new Promise<boolean | string | null>((resolve) => {
        setDialog({ kind, title, body, resolve });
      }),
    [],
  );

  const ctx: BridgeContext = useMemo(
    () => ({
      pluginId: contribution.id,
      paneId,
      locale,
      theme,
      tokens,
      appVersion: "0.2.20",
      permissions: contribution.permissions,
      sessions: {
        compose: async ({ title, skill, prompt }) => {
          const out = await composeNewSession(
            { pluginId: contribution.id, title, skill, prompt },
            {
              createSession: async (t) => {
                const meta = (await api.sessionCreate(undefined, t)) as {
                  id: string;
                };
                return { id: meta.id };
              },
              connectSession: async () => {},
              sendMessage: async () => {},
              saveDraft: (sessionId, text) => {
                saveComposerSessionDraft(sessionId, { text });
              },
            },
          );
          return out;
        },
        run: async ({ title, skill, prompt }) => {
          const out = await runNewSession(
            { pluginId: contribution.id, title, skill, prompt },
            {
              createSession: async (t) => {
                const meta = (await api.sessionCreate(undefined, t)) as {
                  id: string;
                };
                return { id: meta.id };
              },
              connectSession: async (sessionId) => {
                await api.sessionConnect({ sessionId });
              },
              sendMessage: async (sessionId, text) => {
                await api.sessionSend(text, null, sessionId);
              },
              saveDraft: () => {},
            },
          );
          jobsRef.current.set(out.jobId, {
            jobId: out.jobId,
            sessionId: out.sessionId,
            pluginId: contribution.id,
            status: "running",
          });
          postToFrame(sessionStartedPayload(out.jobId, out.sessionId));
          return out;
        },
        open: async () => {},
        get: async () => null,
        poll: async (jobId) => jobsRef.current.get(jobId) ?? null,
      },
      storage: api.isTauri()
        ? {
            get: (key) => api.pluginStorageGet(contribution.id, key),
            set: (key, value) =>
              api.pluginStorageSet(contribution.id, key, value).then(() => undefined),
            list: async () =>
              (await api.pluginStorageList(contribution.id)).keys ?? [],
            delete: (key) =>
              api.pluginStorageDelete(contribution.id, key).then(() => undefined),
          }
        : storageForPlugin(contribution.id),
      dialog: {
        notice: async (title, body) => {
          await openDialog("notice", title, body);
        },
        confirm: async (title, body) =>
          (await openDialog("confirm", title, body)) === true,
        prompt: async (title, body) => {
          const v = await openDialog("prompt", title, body);
          return typeof v === "string" ? v : null;
        },
      },
      toast: () => {},
      clipboardWrite: async (text) => {
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          /* ignore */
        }
      },
      focusPane: () => {},
    }),
    [contribution, locale, openDialog, paneId, postToFrame, theme, tokens],
  );

  useEffect(() => {
    const onMsg = (ev: MessageEvent) => {
      if (origin && ev.origin !== origin) return;
      if (ev.source !== iframeRef.current?.contentWindow) return;
      const req = parseHostReq(ev.data);
      if (!req) return;
      void dispatchHostRequest(req, ctx).then((res) => {
        ev.source?.postMessage(res, { targetOrigin: ev.origin });
      });
    };
    window.addEventListener("message", onMsg);
    return () => window.removeEventListener("message", onMsg);
  }, [ctx, origin]);

  useEffect(() => {
    if (!api.isTauri()) return;
    let cancelled = false;
    const unsubs: Array<() => void> = [];
    void (async () => {
      unsubs.push(
        await api.listen<SessionSnapshot>("session://state", (s) => {
          if (cancelled || !s.sessionId || s.state !== "ready") return;
          for (const [id, job] of jobsRef.current) {
            if (job.sessionId !== s.sessionId || job.status === "done") continue;
            const next = finishJob(job, {
              ok: !s.lastError,
              reason: s.lastError?.message,
              text: "",
            });
            jobsRef.current.set(id, next);
            postToFrame(
              sessionDonePayload({
                jobId: next.jobId,
                sessionId: next.sessionId,
                ok: next.ok ?? true,
                reason: next.reason,
                text: next.text,
              }),
            );
          }
        }),
      );
      unsubs.push(
        await api.listen<{ sessionId?: string }>("session://permission", (p) => {
          if (cancelled) return;
          for (const job of jobsRef.current.values()) {
            if (job.sessionId === p.sessionId && job.status !== "done") {
              postToFrame(sessionNeedsUserPayload(job.jobId, "permission"));
              void openDialog(
                "notice",
                createT(locale)("pluginHost.dialogTitle"),
                createT(locale)("pluginHost.warnTitle"),
              );
            }
          }
        }),
      );
      unsubs.push(
        await api.listen<{ sessionId?: string }>("session://ask_user", (p) => {
          if (cancelled) return;
          for (const job of jobsRef.current.values()) {
            if (job.sessionId === p.sessionId && job.status !== "done") {
              postToFrame(sessionNeedsUserPayload(job.jobId, "ask_user"));
              void openDialog(
                "notice",
                createT(locale)("pluginHost.dialogTitle"),
                createT(locale)("pluginHost.warnTitle"),
              );
            }
          }
        }),
      );
    })();
    return () => {
      cancelled = true;
      unsubs.forEach((u) => u());
    };
  }, [locale, openDialog, postToFrame]);

  if (!pane || !src) {
    return <div className="plugin-pane-host">{tr("pluginHost.empty")}</div>;
  }

  return (
    <div
      className="plugin-pane-host"
      style={{
        display: hidden ? "none" : "flex",
        flex: 1,
        minHeight: 0,
        overflow: "hidden",
      }}
    >
      <iframe
        ref={iframeRef}
        title={pane.titleEn}
        src={src}
        onLoad={() => {
          postToFrame(
            hostReadyPayload({
              pluginId: contribution.id,
              paneId,
              locale,
              theme,
              tokens,
              appVersion: "0.2.20",
              permissions: contribution.permissions,
            }),
          );
        }}
        sandbox="allow-scripts allow-forms allow-same-origin"
        allow=""
        style={{
          border: 0,
          width: "100%",
          height: "100%",
          background: "transparent",
        }}
      />
      <GlassModal
        open={!!dialog}
        onClose={() => {
          dialog?.resolve(dialog.kind === "confirm" ? false : null);
          setDialog(null);
        }}
        title={dialog?.title || tr("pluginHost.dialogTitle")}
        size="sm"
        closeLabel={tr("common.close")}
      >
        <p className="app-dialog__msg">{dialog?.body}</p>
        {dialog?.kind === "prompt" ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              const fd = new FormData(e.currentTarget);
              dialog.resolve(String(fd.get("v") ?? ""));
              setDialog(null);
            }}
          >
            <input name="v" className="c-input" />
            <button type="submit" className="btn btn--solid">
              {tr("common.confirm")}
            </button>
          </form>
        ) : dialog?.kind === "confirm" ? (
          <button
            type="button"
            className="btn btn--solid"
            onClick={() => {
              dialog.resolve(true);
              setDialog(null);
            }}
          >
            {tr("common.confirm")}
          </button>
        ) : null}
      </GlassModal>
    </div>
  );
}
