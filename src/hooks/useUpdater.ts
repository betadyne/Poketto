import { createSignal, onMount } from "solid-js";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateStatus =
    | "idle"
    | "checking"
    | "available"
    | "downloading"
    | "ready"
    | "error"
    | "up-to-date";

export interface UpdateInfo {
    version: string;
    body: string;
    date: string;
}

export function useUpdater() {
    const [status, setStatus] = createSignal<UpdateStatus>("idle");
    const [updateInfo, setUpdateInfo] = createSignal<UpdateInfo | null>(null);
    const [downloadProgress, setDownloadProgress] = createSignal(0);
    const [error, setError] = createSignal<string | null>(null);

    let pendingUpdate: Update | null = null;

    const checkForUpdates = async (silent = false): Promise<boolean> => {
        try {
            if (!silent) {
                setStatus("checking");
            }
            setError(null);

            const update = await check();

            if (update) {
                pendingUpdate = update;
                setUpdateInfo({
                    version: update.version,
                    body: update.body || "No release notes available.",
                    date: update.date || "",
                });
                setStatus("available");
                return true;
            } else {
                if (!silent) {
                    setStatus("up-to-date");
                }
                return false;
            }
        } catch (e) {
            const errorMsg = e instanceof Error ? e.message : String(e);

            if (!silent) {
                setError(errorMsg);
                setStatus("error");
                console.error("Update check failed:", errorMsg);
            }
            return false;
        }
    };

    const downloadAndInstall = async () => {
        if (!pendingUpdate) {
            setError("No update available to download");
            return;
        }

        try {
            setStatus("downloading");
            setDownloadProgress(0);

            let downloaded = 0;
            let totalSize = 0;

            await pendingUpdate.downloadAndInstall((event) => {
                switch (event.event) {
                    case "Started":
                        downloaded = 0;
                        totalSize = (event.data as { contentLength?: number }).contentLength || 0;
                        setDownloadProgress(0);
                        break;
                    case "Progress":
                        const data = event.data as { chunkLength: number; contentLength?: number };
                        downloaded += data.chunkLength;
                        if (totalSize > 0) {
                            setDownloadProgress(Math.round((downloaded / totalSize) * 100));
                        }
                        break;
                    case "Finished":
                        setDownloadProgress(100);
                        break;
                }
            });

            setStatus("ready");
        } catch (e) {
            const errorMsg = e instanceof Error ? e.message : String(e);
            setError(errorMsg);
            setStatus("error");
            console.error("Update download failed:", errorMsg);
        }
    };

    const restartApp = async () => {
        try {
            await relaunch();
        } catch (e) {
            console.error("Failed to restart app:", e);
        }
    };

    const dismissUpdate = () => {
        setStatus("idle");
        setUpdateInfo(null);
        pendingUpdate = null;
    };

    onMount(() => {
        setTimeout(() => {
            checkForUpdates(true);
        }, 3000);
    });

    return {
        status,
        updateInfo,
        downloadProgress,
        error,
        checkForUpdates,
        downloadAndInstall,
        restartApp,
        dismissUpdate,
    };
}
