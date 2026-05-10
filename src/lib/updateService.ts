import { isTauri } from "@tauri-apps/api/core";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type CliplyUpdateInfo = {
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
};

let pendingUpdate: Update | null = null;

export async function checkCliplyUpdate(): Promise<CliplyUpdateInfo | null> {
  if (!isTauri()) {
    return {
      currentVersion: "0.4.0-beta.1",
      version: "0.4.1",
      date: new Date().toISOString(),
      body: "Signed updater preview from the browser mock.",
    };
  }

  pendingUpdate?.close().catch(() => {});
  pendingUpdate = await check();
  if (!pendingUpdate) {
    return null;
  }

  return updateInfoFromPending(pendingUpdate);
}

export async function downloadAndInstallCliplyUpdate(
  onProgress?: (progress: number | null) => void,
): Promise<void> {
  if (!isTauri()) {
    for (const progress of [0.18, 0.42, 0.74, 1]) {
      onProgress?.(progress);
      await new Promise((resolve) => window.setTimeout(resolve, 120));
    }
    return;
  }

  if (!pendingUpdate) {
    pendingUpdate = await check();
  }
  if (!pendingUpdate) {
    throw new Error("当前已经是最新版本");
  }

  let downloaded = 0;
  let total: number | null = null;
  await pendingUpdate.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      downloaded = 0;
      total = event.data.contentLength ?? null;
      onProgress?.(total ? 0 : null);
      return;
    }

    if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress?.(total ? Math.min(downloaded / total, 1) : null);
      return;
    }

    onProgress?.(1);
  });
}

export async function relaunchCliply(): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await relaunch();
}

function updateInfoFromPending(update: Update): CliplyUpdateInfo {
  return {
    currentVersion: update.currentVersion,
    version: update.version,
    date: update.date,
    body: update.body,
  };
}
