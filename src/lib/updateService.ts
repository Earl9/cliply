import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";

export const CLIPLY_RELEASE_PAGE_URL = "https://github.com/Earl9/cliply/releases/latest";
const CLIPLY_UPDATE_MANIFEST_URL =
  "https://github.com/Earl9/cliply/releases/latest/download/latest.json";

export type CliplyUpdateInfo = {
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
  installer: {
    name: string;
    url: string;
    sha256: string;
  };
};

type ModernInstallerManifest = {
  version?: string;
  notes?: string;
  pub_date?: string;
  published_at?: string;
  name?: string;
  url?: string;
  sha256?: string;
};

type CliplyUpdateManifest = {
  version?: string;
  notes?: string;
  pub_date?: string;
  published_at?: string;
  modernInstaller?: ModernInstallerManifest;
  modern_installer?: ModernInstallerManifest;
  installer?: ModernInstallerManifest;
};

type DownloadProgressEvent = {
  downloadedBytes: number;
  totalBytes?: number | null;
};

type ModernInstallerDownloadResult = {
  path: string;
  sha256: string;
  sizeBytes: number;
};

let pendingUpdate: CliplyUpdateInfo | null = null;
let downloadedInstallerPath: string | null = null;

export async function checkCliplyUpdate(): Promise<CliplyUpdateInfo | null> {
  const currentVersion = await getCurrentVersion();
  const manifest = await fetchUpdateManifest();
  const installer = getModernInstallerManifest(manifest);
  const latestVersion = normalizeVersion(installer?.version ?? manifest.version);
  if (!latestVersion) {
    throw new Error("更新清单缺少版本号");
  }

  if (compareSemver(currentVersion, latestVersion) >= 0) {
    pendingUpdate = null;
    downloadedInstallerPath = null;
    return null;
  }

  pendingUpdate = {
    currentVersion,
    version: latestVersion,
    date: installer?.published_at ?? installer?.pub_date ?? manifest.published_at ?? manifest.pub_date,
    body: installer?.notes ?? manifest.notes,
    installer: normalizeModernInstaller(manifest),
  };
  downloadedInstallerPath = null;
  return pendingUpdate;
}

export async function downloadCliplyUpdate(
  onProgress?: (progress: number | null) => void,
): Promise<void> {
  if (!pendingUpdate) {
    const update = await checkCliplyUpdate();
    if (!update) {
      throw new Error("当前已经是最新版本");
    }
  }

  if (!isTauri()) {
    for (const progress of [0.18, 0.42, 0.74, 1]) {
      onProgress?.(progress);
      await new Promise((resolve) => window.setTimeout(resolve, 120));
    }
    downloadedInstallerPath = "browser-mock-modern-installer.exe";
    return;
  }

  const update = pendingUpdate;
  if (!update) {
    throw new Error("当前已经是最新版本");
  }

  const unlisten = await listen<DownloadProgressEvent>(
    "modern-update-download-progress",
    (event) => {
      const total = event.payload.totalBytes ?? null;
      if (!total) {
        onProgress?.(null);
        return;
      }
      onProgress?.(Math.min(event.payload.downloadedBytes / total, 1));
    },
  );

  try {
    const result = await invoke<ModernInstallerDownloadResult>("download_modern_update_installer", {
      request: {
        url: update.installer.url,
        sha256: update.installer.sha256,
        fileName: update.installer.name,
      },
    });
    downloadedInstallerPath = result.path;
    onProgress?.(1);
  } catch (error) {
    downloadedInstallerPath = null;
    throw new Error(errorMessage(error, "更新包校验失败"));
  } finally {
    unlisten();
  }
}

export async function launchModernUpdateInstaller(): Promise<void> {
  if (!pendingUpdate) {
    throw new Error("请先检查更新");
  }
  if (!downloadedInstallerPath) {
    throw new Error("请先下载更新包");
  }

  if (!isTauri()) {
    await new Promise((resolve) => window.setTimeout(resolve, 240));
    return;
  }

  try {
    const installDir = await invoke<string>("get_current_install_dir");
    await invoke<void>("launch_modern_update_installer", {
      request: {
        installerPath: downloadedInstallerPath,
        installDir,
        sourceVersion: pendingUpdate.currentVersion,
        targetVersion: pendingUpdate.version,
      },
    });
  } catch (error) {
    await logUpdateInstallFailed(pendingUpdate.version).catch(() => {});
    throw error;
  }
}

export async function openCliplyReleasePage(): Promise<void> {
  if (!isTauri()) {
    window.open(CLIPLY_RELEASE_PAGE_URL, "_blank", "noopener,noreferrer");
    return;
  }

  await invoke<void>("open_cliply_release_page");
}

async function fetchUpdateManifest(): Promise<CliplyUpdateManifest> {
  if (isTauri()) {
    try {
      return await invoke<CliplyUpdateManifest>("fetch_cliply_update_manifest");
    } catch (error) {
      throw new Error(errorMessage(error, "检查更新失败，请检查网络后重试"));
    }
  }

  const response = await fetch(CLIPLY_UPDATE_MANIFEST_URL, { cache: "no-store" });
  if (!response.ok) {
    throw new Error("无法获取更新清单");
  }
  return (await response.json()) as CliplyUpdateManifest;
}

async function getCurrentVersion(): Promise<string> {
  if (!isTauri()) {
    return "0.4.1-beta.6";
  }
  return getVersion();
}

function normalizeModernInstaller(manifest: CliplyUpdateManifest): CliplyUpdateInfo["installer"] {
  const installer = getModernInstallerManifest(manifest);
  if (!installer?.url || !installer.sha256) {
    throw new Error("更新清单缺少 Modern Installer 信息");
  }
  const name = installer.name || fileNameFromUrl(installer.url);
  if (!name.toLowerCase().endsWith("-modern-installer.exe")) {
    throw new Error("更新清单中的安装器不是 Modern Installer");
  }

  return {
    name,
    url: installer.url,
    sha256: installer.sha256.trim().toLowerCase(),
  };
}

function getModernInstallerManifest(
  manifest: CliplyUpdateManifest,
): ModernInstallerManifest | undefined {
  return manifest.modernInstaller ?? manifest.modern_installer ?? manifest.installer;
}

function normalizeVersion(value?: string | null): string | null {
  const version = value?.trim().replace(/^v/i, "");
  return version || null;
}

function fileNameFromUrl(value: string): string {
  try {
    const url = new URL(value);
    const name = url.pathname.split("/").filter(Boolean).pop();
    return name || "cliply-modern-installer.exe";
  } catch {
    return "cliply-modern-installer.exe";
  }
}

function compareSemver(left: string, right: string): number {
  const leftParts = parseSemver(left);
  const rightParts = parseSemver(right);
  for (let index = 0; index < 3; index += 1) {
    const diff = leftParts.core[index] - rightParts.core[index];
    if (diff !== 0) {
      return diff;
    }
  }
  if (leftParts.pre === rightParts.pre) {
    return 0;
  }
  if (!leftParts.pre) {
    return 1;
  }
  if (!rightParts.pre) {
    return -1;
  }
  return leftParts.pre.localeCompare(rightParts.pre, undefined, { numeric: true });
}

function parseSemver(value: string) {
  const [coreValue, pre = ""] = value.replace(/^v/i, "").split("-");
  const core = coreValue.split(".").map((part) => Number.parseInt(part, 10) || 0);
  return {
    core: [core[0] ?? 0, core[1] ?? 0, core[2] ?? 0] as [number, number, number],
    pre,
  };
}

async function logUpdateInstallFailed(version: string): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke<void>("log_update_install_failed", { version });
}

function errorMessage(error: unknown, fallback: string) {
  if (typeof error === "string") {
    return error || fallback;
  }
  if (error instanceof Error) {
    return error.message || fallback;
  }
  return fallback;
}
