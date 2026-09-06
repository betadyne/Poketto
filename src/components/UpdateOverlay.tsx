import { Match, Show, Switch } from "solid-js";
import {
  IconX,
  IconDownload,
  IconRefresh,
  IconCircleCheck,
  IconAlertCircle,
  IconExternalLink,
} from "@tabler/icons-solidjs";
import type { UpdateStatus, UpdateInfo } from "../hooks/useUpdater";

interface UpdateOverlayProps {
  status: UpdateStatus;
  updateInfo: UpdateInfo | null;
  downloadProgress: number;
  error: string | null;
  onDownload: () => void;
  onManualDownload: () => void;
  onRetry: () => void;
  onRestart: () => void;
  onDismiss: () => void;
}

export function UpdateOverlay(props: UpdateOverlayProps) {
  return (
    <Show when={props.status !== "idle"}>
      <div class="fixed bottom-4 right-4 z-[70] w-[min(92vw,380px)] bg-[var(--color-bg-primary)] border border-[var(--color-border)] rounded-xl shadow-xl">
        <div class="flex items-center gap-2 px-4 py-3 border-b border-[var(--color-border)]">
          <Switch>
            <Match when={props.status === "downloading" || props.status === "checking"}>
              <IconRefresh class="w-5 h-5 text-[var(--color-accent)] animate-spin" strokeWidth={1.5} />
            </Match>
            <Match when={props.status === "ready" || props.status === "up-to-date"}>
              <IconCircleCheck class="w-5 h-5 text-[var(--color-success)]" strokeWidth={1.5} />
            </Match>
            <Match when={props.status === "manual" || props.status === "error"}>
              <IconAlertCircle class="w-5 h-5 text-[var(--color-danger)]" strokeWidth={1.5} />
            </Match>
          </Switch>
          <h2 class="text-sm font-bold text-[var(--color-text-primary)] flex-1">
            <Switch fallback="Update">
              <Match when={props.status === "available"}>Update Available</Match>
              <Match when={props.status === "checking"}>Checking for Updates</Match>
              <Match when={props.status === "downloading"}>Downloading Update</Match>
              <Match when={props.status === "ready"}>Update Ready</Match>
              <Match when={props.status === "manual"}>Manual Update Required</Match>
              <Match when={props.status === "error"}>Update Error</Match>
              <Match when={props.status === "up-to-date"}>Up to Date</Match>
            </Switch>
          </h2>
          <button
            onClick={props.onDismiss}
            class="text-[var(--color-icon)] hover:text-[var(--color-text-primary)]"
          >
            <IconX class="w-5 h-5" strokeWidth={1.5} />
          </button>
        </div>

        <div class="p-4 space-y-3">
          <Show when={props.status === "available" && props.updateInfo}>
            <p class="text-sm text-[var(--color-text-secondary)]">
              Version{" "}
              <span class="text-[var(--color-text-primary)] font-semibold">
                v{props.updateInfo!.version}
              </span>{" "}
              is available!
            </p>
            <div class="bg-[var(--color-bg-secondary)] rounded-lg p-3 max-h-40 overflow-y-auto">
              <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap">
                {props.updateInfo!.body}
              </p>
            </div>
            <div class="flex flex-col sm:flex-row gap-2">
              <button
                onClick={props.onDownload}
                class="flex-1 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] rounded-lg text-white text-sm font-medium flex items-center justify-center gap-2"
              >
                <IconDownload class="w-4 h-4" strokeWidth={1.5} />
                Download & Install
              </button>
              <button
                onClick={props.onDismiss}
                class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] text-sm"
              >
                Later
              </button>
            </div>
          </Show>

          <Show when={props.status === "checking"}>
            <p class="text-sm text-[var(--color-text-secondary)]">
              Contacting the update server...
            </p>
          </Show>

          <Show when={props.status === "downloading"}>
            <div class="w-full bg-[var(--color-bg-secondary)] rounded-full h-2">
              <div
                class="bg-[var(--color-accent)] h-2 rounded-full transition-all duration-300"
                style={{ width: `${props.downloadProgress}%` }}
              />
            </div>
            <p class="text-sm text-[var(--color-text-tertiary)] text-center">
              {Math.round(props.downloadProgress)}%
            </p>
          </Show>

          <Show when={props.status === "ready"}>
            <p class="text-sm text-[var(--color-text-secondary)]">
              Update downloaded! Restart to apply it.
            </p>
            <div class="flex flex-col sm:flex-row gap-2">
              <button
                onClick={props.onRestart}
                class="flex-1 px-4 py-2 bg-[var(--color-success)] hover:opacity-90 rounded-lg text-white text-sm font-medium flex items-center justify-center gap-2"
              >
                <IconRefresh class="w-4 h-4" strokeWidth={1.5} />
                Restart Now
              </button>
              <button
                onClick={props.onDismiss}
                class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] text-sm"
              >
                Later
              </button>
            </div>
          </Show>

          <Show when={props.status === "manual" && props.updateInfo}>
            <p class="text-sm text-[var(--color-text-secondary)]">
              Version{" "}
              <span class="text-[var(--color-text-primary)] font-semibold">
                v{props.updateInfo!.version}
              </span>{" "}
              is available, but automatic install is not supported for this
              install type. Grab the package from the release page.
            </p>
            <div class="flex flex-col sm:flex-row gap-2">
              <button
                onClick={props.onManualDownload}
                class="flex-1 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] rounded-lg text-white text-sm font-medium flex items-center justify-center gap-2"
              >
                <IconExternalLink class="w-4 h-4" strokeWidth={1.5} />
                Open Release Page
              </button>
              <button
                onClick={props.onDismiss}
                class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] text-sm"
              >
                Later
              </button>
            </div>
          </Show>

          <Show when={props.status === "error"}>
            <p class="text-sm text-[var(--color-danger)]">
              Update failed{props.error ? `: ${props.error}` : "."} Check the
              Logs page for details.
            </p>
            <div class="flex flex-col sm:flex-row gap-2">
              <button
                onClick={props.onRetry}
                class="flex-1 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] rounded-lg text-white text-sm font-medium flex items-center justify-center gap-2"
              >
                <IconRefresh class="w-4 h-4" strokeWidth={1.5} />
                Retry
              </button>
              <button
                onClick={props.onManualDownload}
                class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] text-sm flex items-center justify-center gap-2"
              >
                <IconExternalLink class="w-4 h-4" strokeWidth={1.5} />
                Release Page
              </button>
            </div>
          </Show>

          <Show when={props.status === "up-to-date"}>
            <p class="text-sm text-[var(--color-text-secondary)]">
              You're already on the latest version.
            </p>
            <button
              onClick={props.onDismiss}
              class="w-full px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] text-sm"
            >
              Close
            </button>
          </Show>
        </div>
      </div>
    </Show>
  );
}
