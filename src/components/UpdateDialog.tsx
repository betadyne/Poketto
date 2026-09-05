import { Show } from "solid-js";
import { IconX, IconDownload, IconRefresh, IconCircleCheck, IconAlertCircle } from "@tabler/icons-solidjs";
import type { UpdateStatus, UpdateInfo } from "../hooks/useUpdater";

interface UpdateDialogProps {
  status: UpdateStatus;
  updateInfo: UpdateInfo | null;
  downloadProgress: number;
  error: string | null;
  onDownload: () => void;
  onRestart: () => void;
  onDismiss: () => void;
}

export function UpdateDialog(props: UpdateDialogProps) {
  const showDialog = () =>
    props.status === "available" ||
    props.status === "downloading" ||
    props.status === "ready" ||
    props.status === "error";

  return (
    <Show when={showDialog()}>
      <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60] p-4"
        onClick={props.onDismiss}
      >
        <div
          class="bg-[var(--color-bg-primary)] rounded-lg w-full max-w-md shadow-xl"
          onClick={(e) => e.stopPropagation()}
        >
          <div class="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
            <h2 class="text-lg font-bold text-[var(--color-text-primary)] flex items-center gap-2">
              <Show when={props.status === "available"}>
                <IconDownload class="w-5 h-5 text-[var(--color-accent)]" strokeWidth={1.5} />
                Update Available
              </Show>
              <Show when={props.status === "downloading"}>
                <IconRefresh class="w-5 h-5 text-[var(--color-accent)] animate-spin" strokeWidth={1.5} />
                Downloading Update
              </Show>
              <Show when={props.status === "ready"}>
                <IconCircleCheck class="w-5 h-5 text-[var(--color-success)]" strokeWidth={1.5} />
                Update Ready
              </Show>
              <Show when={props.status === "error"}>
                <IconAlertCircle class="w-5 h-5 text-[var(--color-danger)]" strokeWidth={1.5} />
                Update Error
              </Show>
            </h2>
            <button
              onClick={props.onDismiss}
              class="text-[var(--color-icon)] hover:text-[var(--color-text-primary)]"
            >
              <IconX class="w-5 h-5" strokeWidth={1.5} />
            </button>
          </div>

          <div class="p-4 space-y-4">
            <Show when={props.status === "available" && props.updateInfo}>
              <div class="space-y-3">
                <p class="text-[var(--color-text-secondary)]">
                  A new version{" "}
                  <span class="text-[var(--color-text-primary)] font-semibold">
                    v{props.updateInfo!.version}
                  </span>{" "}
                  is available!
                </p>
                <div class="bg-[var(--color-bg-secondary)] rounded-lg p-3 max-h-40 overflow-y-auto">
                  <h4 class="text-sm font-medium text-[var(--color-text-primary)] mb-2">
                    Release Notes:
                  </h4>
                  <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap">
                    {props.updateInfo!.body}
                  </p>
                </div>
                <div class="flex gap-3 pt-2">
                  <button
                    onClick={props.onDownload}
                    class="flex-1 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] rounded-lg text-white font-medium flex items-center justify-center gap-2"
                  >
                    <IconDownload class="w-4 h-4" strokeWidth={1.5} />
                    Download & Install
                  </button>
                  <button
                    onClick={props.onDismiss}
                    class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)]"
                  >
                    Later
                  </button>
                </div>
              </div>
            </Show>

            <Show when={props.status === "downloading"}>
              <div class="space-y-3">
                <p class="text-[var(--color-text-secondary)]">Downloading update...</p>
                <div class="w-full bg-[var(--color-bg-secondary)] rounded-full h-2">
                  <div
                    class="bg-[var(--color-accent)] h-2 rounded-full transition-all duration-300"
                    style={{ width: `${props.downloadProgress}%` }}
                  />
                </div>
                <p class="text-sm text-[var(--color-text-tertiary)] text-center">
                  {Math.round(props.downloadProgress)}%
                </p>
              </div>
            </Show>

            <Show when={props.status === "ready"}>
              <div class="space-y-3">
                <p class="text-[var(--color-text-secondary)]">
                  Update downloaded successfully! Restart to apply the update.
                </p>
                <div class="flex gap-3 pt-2">
                  <button
                    onClick={props.onRestart}
                    class="flex-1 px-4 py-2 bg-[var(--color-success)] hover:opacity-90 rounded-lg text-white font-medium flex items-center justify-center gap-2"
                  >
                    <IconRefresh class="w-4 h-4" strokeWidth={1.5} />
                    Restart Now
                  </button>
                  <button
                    onClick={props.onDismiss}
                    class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)]"
                  >
                    Later
                  </button>
                </div>
              </div>
            </Show>

            <Show when={props.status === "error"}>
              <div class="space-y-3">
                <p class="text-[var(--color-danger)]">
                  Failed to check for updates: {props.error}
                </p>
                <button
                  onClick={props.onDismiss}
                  class="w-full px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)]"
                >
                  Close
                </button>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
