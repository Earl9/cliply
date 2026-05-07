import {
  useEffect,
  useState,
  type Dispatch,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  type SetStateAction,
} from "react";
import { open as openFileDialog, save as saveFileDialog } from "@tauri-apps/plugin-dialog";
import {
  BellOff,
  Check,
  CircleHelp,
  History,
  Keyboard,
  Monitor,
  Moon,
  Palette,
  RefreshCw,
  Settings2,
  Shield,
  Sparkles,
  Sun,
  X,
  type LucideIcon,
} from "lucide-react";
import { clsx } from "clsx";
import { Badge } from "@/components/common/Badge";
import { IconButton } from "@/components/common/IconButton";
import { getCliplyDebugInfo, type CliplyDebugInfo } from "@/lib/debugInfo";
import {
  checkGlobalShortcut,
  clearAutoSyncPassword,
  exportSyncPackage,
  exportToRemoteSyncFolder,
  getRemoteSyncStatus,
  getSyncPackageStatus,
  importFromRemoteSyncFolder,
  importSyncPackage,
  setRemoteSyncProvider,
  syncWithRemoteNow,
  updateAutoSyncConfig,
  type RemoteSyncResult,
  type RemoteSyncStatus,
  type SyncImportResult,
  type SyncPackageStatus,
  type SyncProviderConfig,
  type ShortcutCheck,
} from "@/lib/settingsRepository";
import {
  CLIPLY_THEME_OPTIONS,
  DEFAULT_THEME_NAME,
  applyCliplyTheme,
  getCliplyTheme,
  getCliplyThemeWithAccent,
  isCliplyThemeName,
  type CliplyThemeName,
} from "@/theme/theme";
import type { CliplySettings } from "@/stores/settingsStore";

type SettingsDialogProps = {
  open: boolean;
  settings: CliplySettings;
  onClose: () => void;
  onSave: (settings: CliplySettings) => void;
  onClearHistory: () => void;
};

type FtpProviderConfig = Extract<SyncProviderConfig, { type: "ftp" }>;
type WebdavProviderConfig = Extract<SyncProviderConfig, { type: "webdav" }>;
type SettingsTab =
  | "general"
  | "shortcuts"
  | "privacy"
  | "history"
  | "appearance"
  | "sync"
  | "about";
type UpdateSettingsDraft = <K extends keyof CliplySettings>(
  key: K,
  value: CliplySettings[K],
) => void;

const SYNC_PROVIDER_OPTIONS: Array<{
  type: "disabled" | "local-folder" | "webdav" | "ftp";
  label: string;
}> = [
  { type: "disabled", label: "关闭同步" },
  { type: "local-folder", label: "本地文件夹" },
  { type: "webdav", label: "WebDAV" },
  { type: "ftp", label: "FTP/FTPS" },
];

const SETTINGS_TABS: Array<{
  id: SettingsTab;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  { id: "general", label: "通用", description: "启动、监听和窗口行为。", icon: Settings2 },
  { id: "shortcuts", label: "快捷键", description: "打开窗口和列表内操作快捷键。", icon: Keyboard },
  { id: "privacy", label: "隐私", description: "敏感内容、图片和忽略应用。", icon: Shield },
  { id: "history", label: "历史记录", description: "容量、清理和重复内容策略。", icon: History },
  { id: "appearance", label: "外观", description: "主题方案和当前视觉预览。", icon: Sparkles },
  { id: "sync", label: "同步", description: "加密同步包、远程目录和自动同步。", icon: RefreshCw },
  { id: "about", label: "关于", description: "版本、数据目录和调试信息。", icon: CircleHelp },
];

const CLIPLY_VERSION = "0.1.0";
const ACCENT_PRESET_COLORS = [
  "#6D4CFF",
  "#3B82F6",
  "#14B8A6",
  "#22C55E",
  "#F97316",
  "#E856B6",
  "#0EA5E9",
  "#111827",
];
const THEME_SUMMARIES: Record<CliplyThemeName, string> = {
  "purple-default": "现代、稳定",
  "lake-blue": "克制、专业",
  "teal-fresh": "科技、隐私",
  "mint-green": "清新、轻盈",
  "coral-orange": "活泼、个性",
  "rose-violet": "柔和、精致",
};

let sessionSyncPassword = "";

export function SettingsDialog({
  open,
  settings,
  onClose,
  onSave,
  onClearHistory,
}: SettingsDialogProps) {
  const [draft, setDraft] = useState(settings);
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [capturingShortcut, setCapturingShortcut] = useState(false);
  const [shortcutCheck, setShortcutCheck] = useState<ShortcutCheck | null>(null);
  const [debugInfo, setDebugInfo] = useState<CliplyDebugInfo | null>(null);
  const [syncPassword, setSyncPassword] = useState(sessionSyncPassword);
  const [autoSyncEnabled, setAutoSyncEnabled] = useState(false);
  const [autoSyncIntervalMinutes, setAutoSyncIntervalMinutes] = useState(5);
  const [syncStatus, setSyncStatus] = useState<SyncPackageStatus>({});
  const [remoteSyncStatus, setRemoteSyncStatus] = useState<RemoteSyncStatus>({
    provider: { type: "disabled" },
    savedProviderConfigs: {},
    manifestExists: false,
    snapshotCount: 0,
    autoSyncEnabled: false,
    autoSyncIntervalMinutes: 5,
    syncPasswordSaved: false,
  });
  const [savedSyncProvider, setSavedSyncProvider] = useState<SyncProviderConfig>({
    type: "disabled",
  });
  const [webdavDraft, setWebdavDraft] = useState<WebdavProviderConfig>(defaultWebdavConfig());
  const [ftpDraft, setFtpDraft] = useState<FtpProviderConfig>(defaultFtpConfig());
  const [selectedSyncProviderType, setSelectedSyncProviderType] =
    useState<SyncProviderConfig["type"]>("disabled");
  const [syncBusy, setSyncBusy] = useState<"export" | "import" | "sync" | null>(null);
  const [providerBusy, setProviderBusy] = useState(false);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);
  const ignoreAppsText = draft.ignoreApps.join("\n");

  useEffect(() => {
    if (open) {
      setDraft(settings);
      setCapturingShortcut(false);
      setShortcutCheck(null);
      setSyncPassword(sessionSyncPassword);
      setSyncMessage(null);
      setSyncError(null);
      void refreshSyncStatus();
    }
  }, [open, settings]);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;
    void getCliplyDebugInfo()
      .then((info) => {
        if (!cancelled) {
          setDebugInfo(info);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDebugInfo(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [open]);

  useEffect(() => {
    if (open) {
      applyCliplyTheme(getCliplyThemeWithAccent(getDraftThemeName(draft.themeName), draft.accentColor));
    }
  }, [draft.accentColor, draft.themeName, open]);

  useEffect(() => {
    setSavedSyncProvider(remoteSyncStatus.provider);
    setSelectedSyncProviderType(
      remoteSyncStatus.provider.type === "local-folder" ||
        remoteSyncStatus.provider.type === "webdav" ||
        remoteSyncStatus.provider.type === "ftp" ||
        remoteSyncStatus.provider.type === "disabled"
        ? remoteSyncStatus.provider.type
        : "disabled",
    );
    setAutoSyncEnabled(remoteSyncStatus.autoSyncEnabled);
    setAutoSyncIntervalMinutes(remoteSyncStatus.autoSyncIntervalMinutes || 5);
    if (remoteSyncStatus.savedProviderConfigs.webdav) {
      setWebdavDraft(normalizeWebdavConfig(remoteSyncStatus.savedProviderConfigs.webdav));
    }
    if (remoteSyncStatus.savedProviderConfigs.ftp) {
      setFtpDraft(normalizeFtpConfig(remoteSyncStatus.savedProviderConfigs.ftp));
    }
    if (remoteSyncStatus.provider.type === "webdav") {
      setWebdavDraft(normalizeWebdavConfig(remoteSyncStatus.provider));
    }
    if (remoteSyncStatus.provider.type === "ftp") {
      setFtpDraft(normalizeFtpConfig(remoteSyncStatus.provider));
    }
  }, [
    remoteSyncStatus.autoSyncEnabled,
    remoteSyncStatus.autoSyncIntervalMinutes,
    remoteSyncStatus.provider,
    remoteSyncStatus.savedProviderConfigs.ftp,
    remoteSyncStatus.savedProviderConfigs.webdav,
  ]);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;
    const timeout = window.setTimeout(() => {
      void checkGlobalShortcut(draft.globalShortcut, settings.globalShortcut)
        .then((result) => {
          if (cancelled) {
            return;
          }

          setShortcutCheck(result);
          if (result.ok && result.display && result.display !== draft.globalShortcut) {
            setDraft((current) => ({
              ...current,
              globalShortcut: result.display,
            }));
          }
        })
        .catch((error) => {
          if (cancelled) {
            return;
          }

          setShortcutCheck({
            ok: false,
            normalized: "",
            display: draft.globalShortcut,
            reason: "system-conflict",
            message: error instanceof Error ? error.message : "快捷键检测失败",
          });
        });
    }, 220);

    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
    };
  }, [draft.globalShortcut, open, settings.globalShortcut]);

  if (!open) {
    return null;
  }

  const updateDraft = <K extends keyof CliplySettings>(key: K, value: CliplySettings[K]) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  const cancelSettings = () => {
    applyCliplyTheme(
      getCliplyThemeWithAccent(getDraftThemeName(settings.themeName), settings.accentColor),
    );
    setCapturingShortcut(false);
    onClose();
  };

  const saveDisabled = !shortcutCheck?.ok;
  const syncActionDisabled = syncBusy !== null || !syncPassword.trim();
  const remoteSyncActionDisabled =
    syncBusy !== null || (!syncPassword.trim() && !remoteSyncStatus.syncPasswordSaved);

  const refreshSyncStatus = async () => {
    try {
      const [packageStatus, remoteStatus] = await Promise.all([
        getSyncPackageStatus(),
        getRemoteSyncStatus(),
      ]);
      setSyncStatus(packageStatus);
      setRemoteSyncStatus(remoteStatus);
      if (remoteStatus.lastError) {
        setSyncError(remoteStatus.lastError);
      }
    } catch {
      setSyncStatus({});
      setSyncError("同步配置读取失败");
    }
  };

  const handleSyncProviderChange = async (type: SyncProviderConfig["type"]) => {
    setSelectedSyncProviderType(type);
    if (type === "webdav") {
      const cachedWebdav = remoteSyncStatus.savedProviderConfigs.webdav;
      const nextWebdav =
        savedSyncProvider.type === "webdav"
          ? normalizeWebdavConfig(savedSyncProvider)
          : cachedWebdav
            ? normalizeWebdavConfig(cachedWebdav)
            : hasWebdavDraft(webdavDraft)
              ? webdavDraft
              : defaultWebdavConfig();
      setWebdavDraft(nextWebdav);
      setSyncMessage("请填写 WebDAV 信息后点击保存");
      setSyncError(null);
      return;
    }

    if (type === "ftp") {
      const cachedFtp = remoteSyncStatus.savedProviderConfigs.ftp;
      const nextFtp =
        savedSyncProvider.type === "ftp"
          ? normalizeFtpConfig(savedSyncProvider)
          : cachedFtp
            ? normalizeFtpConfig(cachedFtp)
            : hasFtpDraft(ftpDraft)
              ? ftpDraft
              : defaultFtpConfig();
      setFtpDraft(nextFtp);
      setSyncMessage("请填写 FTP/FTPS 信息后点击保存");
      setSyncError(null);
      return;
    }

    if (type === "local-folder") {
      const nextProvider =
        savedSyncProvider.type === "local-folder"
          ? savedSyncProvider
          : (remoteSyncStatus.savedProviderConfigs.localFolder ?? {
              type: "local-folder",
              path: "",
            } as const);
      if (!nextProvider.path) {
        setSyncMessage("请点击“选择文件夹”设置本地同步目录");
        setSyncError(null);
        return;
      }
      try {
        const status = await setRemoteSyncProvider(nextProvider);
        setRemoteSyncStatus(status);
        setSavedSyncProvider(status.provider);
        setSyncMessage("本地同步文件夹已启用");
        setSyncError(null);
      } catch (error) {
        setSyncError(errorMessage(error, "本地同步文件夹启用失败"));
      }
      return;
    }

    const nextProvider = { type: "disabled" as const };

    try {
      const status = await setRemoteSyncProvider(nextProvider);
      setRemoteSyncStatus(status);
      setSavedSyncProvider(status.provider);
      setSyncMessage("同步已关闭");
      setSyncError(null);
    } catch (error) {
      setSyncError(errorMessage(error, "同步方式保存失败"));
    }
  };

  const handleChooseSyncFolder = async () => {
    const selectedPath = await openFileDialog({
      title: "选择 Cliply 同步文件夹",
      directory: true,
      multiple: false,
    });
    if (!selectedPath || Array.isArray(selectedPath)) {
      return;
    }

    setProviderBusy(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const status = await setRemoteSyncProvider({ type: "local-folder", path: selectedPath });
      setRemoteSyncStatus(status);
      setSavedSyncProvider(status.provider);
      setSelectedSyncProviderType("local-folder");
      setSyncMessage("本地同步文件夹已设置");
    } catch (error) {
      setSyncError(errorMessage(error, "同步文件夹设置失败"));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleSaveWebdavProvider = async () => {
    const nextConfig = normalizeWebdavConfig(webdavDraft);
    if (!nextConfig.url.trim() || !nextConfig.username.trim() || !nextConfig.password) {
      setSyncMessage(null);
      setSyncError("请填写 WebDAV 地址、用户名和密码");
      return;
    }
    if (!/^https?:\/\//i.test(nextConfig.url)) {
      setSyncMessage(null);
      setSyncError("WebDAV 地址必须以 http:// 或 https:// 开头");
      return;
    }

    setProviderBusy(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const status = await setRemoteSyncProvider(nextConfig);
      const savedProvider =
        status.provider.type === "webdav" ? normalizeWebdavConfig(status.provider) : nextConfig;
      setWebdavDraft(savedProvider);
      setRemoteSyncStatus(status);
      setSavedSyncProvider(status.provider);
      setSelectedSyncProviderType("webdav");
      setSyncMessage("WebDAV 配置已保存。导出、导入和自动同步会使用该地址。");
      setSyncError(null);
    } catch (error) {
      setSyncError(errorMessage(error, "WebDAV 同步配置保存失败"));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleSaveFtpProvider = async () => {
    const nextConfig = normalizeFtpConfig(ftpDraft);
    if (!nextConfig.host.trim() || !nextConfig.username.trim() || !nextConfig.password) {
      setSyncMessage(null);
      setSyncError("请填写 FTP 主机、用户名和密码");
      return;
    }

    setProviderBusy(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const status = await setRemoteSyncProvider(nextConfig);
      const savedProvider =
        status.provider.type === "ftp" ? normalizeFtpConfig(status.provider) : nextConfig;
      setFtpDraft(savedProvider);
      setRemoteSyncStatus(status);
      setSavedSyncProvider(status.provider);
      setSelectedSyncProviderType("ftp");
      setSyncMessage(
        `${nextConfig.secure ? "FTPS" : "FTP"} 配置已保存。导出或导入时会连接服务器。`,
      );
      setSyncError(null);
    } catch (error) {
      setSyncError(errorMessage(error, "FTP 同步配置保存失败"));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleExportToRemoteFolder = async () => {
    if (!syncPassword.trim()) {
      setSyncError("请输入同步密码");
      return;
    }

    setSyncBusy("export");
    setSyncMessage(null);
    setSyncError(null);
    try {
      const result = await exportToRemoteSyncFolder(syncPassword);
      setSyncMessage(
        remoteSyncResultMessage(
          result,
          `${remoteSyncProviderLabel(selectedSyncProviderType)}导出完成`,
        ),
      );
      applyRemoteSyncResult(result);
    } catch (error) {
      setSyncError(errorMessage(error, "导出到同步文件夹失败"));
    } finally {
      setSyncBusy(null);
    }
  };

  const handleImportFromRemoteFolder = async () => {
    if (!syncPassword.trim()) {
      setSyncError("请输入同步密码");
      return;
    }

    setSyncBusy("import");
    setSyncMessage(null);
    setSyncError(null);
    try {
      const result = await importFromRemoteSyncFolder(syncPassword);
      setSyncMessage(
        remoteSyncResultMessage(
          result,
          `${remoteSyncProviderLabel(selectedSyncProviderType)}导入完成`,
        ),
      );
      applyRemoteSyncResult(result);
    } catch (error) {
      setSyncError(errorMessage(error, "从同步文件夹导入失败"));
    } finally {
      setSyncBusy(null);
    }
  };

  const handleSaveAutoSync = async () => {
    if (autoSyncEnabled && selectedSyncProviderType === "disabled") {
      setSyncMessage(null);
      setSyncError("请先选择本地文件夹、WebDAV 或 FTP/FTPS");
      return;
    }

    setProviderBusy(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const status = await updateAutoSyncConfig(
        autoSyncEnabled,
        autoSyncIntervalMinutes,
        syncPassword,
      );
      setRemoteSyncStatus(status);
      setSyncMessage(autoSyncEnabled ? "自动同步已开启" : "自动同步已关闭");
    } catch (error) {
      setSyncError(errorMessage(error, "自动同步配置保存失败"));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleClearAutoSyncPassword = async () => {
    setProviderBusy(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const status = await clearAutoSyncPassword();
      setRemoteSyncStatus(status);
      setAutoSyncEnabled(false);
      setSyncMessage("已保存的同步密码已清除，自动同步已关闭");
    } catch (error) {
      setSyncError(errorMessage(error, "清除同步密码失败"));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleSyncWithRemoteNow = async () => {
    if (!syncPassword.trim() && !remoteSyncStatus.syncPasswordSaved) {
      setSyncError("请输入同步密码，或先保存同步密码");
      return;
    }

    setSyncBusy("sync");
    setSyncMessage(null);
    setSyncError(null);
    try {
      const result = await syncWithRemoteNow(syncPassword);
      setSyncMessage(remoteSyncResultMessage(result, "同步完成"));
      applyRemoteSyncResult(result);
      await refreshSyncStatus();
    } catch (error) {
      setSyncError(errorMessage(error, "同步失败"));
    } finally {
      setSyncBusy(null);
    }
  };

  const handleExportSyncPackage = async () => {
    if (!syncPassword.trim()) {
      setSyncError("请输入同步密码");
      return;
    }

    const selectedPath = await saveFileDialog({
      title: "导出 Cliply 同步包",
      defaultPath: `Cliply-${new Date().toISOString().slice(0, 10)}.cliply-sync`,
      filters: [{ name: "Cliply Sync Package", extensions: ["cliply-sync"] }],
    });
    if (!selectedPath) {
      return;
    }

    setSyncBusy("export");
    setSyncMessage(null);
    setSyncError(null);
    try {
      await exportSyncPackage(selectedPath, syncPassword);
      setSyncMessage("同步包已导出");
      await refreshSyncStatus();
    } catch (error) {
      setSyncError(errorMessage(error, "同步包导出失败"));
    } finally {
      setSyncBusy(null);
    }
  };

  const applyRemoteSyncResult = (result: RemoteSyncResult) => {
    setRemoteSyncStatus((current) => ({
      ...current,
      manifestExists: true,
      snapshotCount: result.snapshotCount,
      lastSyncedAt: result.syncedAt,
      lastStatus: "success",
      lastError: "",
    }));
  };

  const handleImportSyncPackage = async () => {
    if (!syncPassword.trim()) {
      setSyncError("请输入同步密码");
      return;
    }

    const selectedPath = await openFileDialog({
      title: "导入 Cliply 同步包",
      multiple: false,
      filters: [{ name: "Cliply Sync Package", extensions: ["cliply-sync"] }],
    });
    if (!selectedPath || Array.isArray(selectedPath)) {
      return;
    }

    setSyncBusy("import");
    setSyncMessage(null);
    setSyncError(null);
    try {
      const result = await importSyncPackage(selectedPath, syncPassword);
      setSyncMessage(syncImportResultMessage(result));
      await refreshSyncStatus();
    } catch (error) {
      setSyncError(errorMessage(error, "导入失败，已回滚"));
    } finally {
      setSyncBusy(null);
    }
  };

  const activeTabMeta = SETTINGS_TABS.find((tab) => tab.id === activeTab) ?? SETTINGS_TABS[0];
  const activeContent = (() => {
    switch (activeTab) {
      case "general":
        return <GeneralSettingsTab draft={draft} updateDraft={updateDraft} />;
      case "shortcuts":
        return (
          <ShortcutsSettingsTab
            value={draft.globalShortcut}
            check={shortcutCheck}
            capturing={capturingShortcut}
            onStartCapture={() => setCapturingShortcut(true)}
            onStopCapture={() => setCapturingShortcut(false)}
            onChange={(value) => updateDraft("globalShortcut", value)}
          />
        );
      case "privacy":
        return (
          <PrivacySettingsTab
            draft={draft}
            ignoreAppsText={ignoreAppsText}
            updateDraft={updateDraft}
          />
        );
      case "history":
        return (
          <HistorySettingsTab
            draft={draft}
            updateDraft={updateDraft}
            onClearHistory={onClearHistory}
          />
        );
      case "appearance":
        return <AppearanceSettingsTab draft={draft} updateDraft={updateDraft} />;
      case "sync":
        return (
          <SyncSettingsTab
            remoteSyncStatus={remoteSyncStatus}
            syncStatus={syncStatus}
            selectedSyncProviderType={selectedSyncProviderType}
            webdavDraft={webdavDraft}
            ftpDraft={ftpDraft}
            syncPassword={syncPassword}
            autoSyncEnabled={autoSyncEnabled}
            autoSyncIntervalMinutes={autoSyncIntervalMinutes}
            syncBusy={syncBusy}
            providerBusy={providerBusy}
            syncMessage={syncMessage}
            syncError={syncError}
            syncActionDisabled={syncActionDisabled}
            remoteSyncActionDisabled={remoteSyncActionDisabled}
            setWebdavDraft={setWebdavDraft}
            setFtpDraft={setFtpDraft}
            setSyncPassword={(value) => {
              sessionSyncPassword = value;
              setSyncPassword(value);
              setSyncError(null);
            }}
            setAutoSyncEnabled={setAutoSyncEnabled}
            setAutoSyncIntervalMinutes={setAutoSyncIntervalMinutes}
            onProviderChange={handleSyncProviderChange}
            onChooseSyncFolder={handleChooseSyncFolder}
            onSaveWebdavProvider={handleSaveWebdavProvider}
            onSaveFtpProvider={handleSaveFtpProvider}
            onSaveAutoSync={handleSaveAutoSync}
            onClearAutoSyncPassword={handleClearAutoSyncPassword}
            onSyncNow={handleSyncWithRemoteNow}
            onExportSyncPackage={handleExportSyncPackage}
            onImportSyncPackage={handleImportSyncPackage}
            onExportToRemote={handleExportToRemoteFolder}
            onImportFromRemote={handleImportFromRemoteFolder}
          />
        );
      case "about":
        return <AboutSettingsTab debugInfo={debugInfo} />;
      default:
        return null;
    }
  })();

  return (
    <div className="absolute inset-0 z-30 grid place-items-center bg-slate-900/18 px-6 backdrop-blur-sm">
      <div className="absolute inset-0" aria-hidden="true" data-tauri-drag-region />
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-settings-title"
        className="relative z-10 flex h-[min(720px,calc(100vh-40px))] w-[min(1040px,calc(100vw-48px))] flex-col overflow-hidden rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] shadow-2xl"
      >
        <header
          className="flex h-16 shrink-0 select-none items-center justify-between border-b border-[color:var(--cliply-border)] bg-white/86 px-6"
          data-tauri-drag-region
        >
          <div data-tauri-drag-region>
            <h2
              id="cliply-settings-title"
              className="text-[16px] font-semibold text-[color:var(--cliply-text)]"
              data-tauri-drag-region
            >
              设置
            </h2>
            <p
              className="mt-1 text-xs font-medium text-[color:var(--cliply-muted)]"
              data-tauri-drag-region
            >
              本地优先，Windows MVP
            </p>
          </div>
          <IconButton label="关闭设置" onClick={cancelSettings}>
            <X className="size-4" />
          </IconButton>
        </header>

        <div className="flex min-h-0 flex-1">
          <aside className="w-[200px] shrink-0 border-r border-[color:var(--cliply-border)] bg-[#fbfcfe] p-3">
            <nav className="grid gap-1" aria-label="设置分类">
              {SETTINGS_TABS.map((tab) => {
                const selected = activeTab === tab.id;
                const Icon = tab.icon;
                return (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setActiveTab(tab.id)}
                    className={clsx(
                      "flex h-11 items-center gap-3 rounded-[10px] px-3 text-left text-[13px] font-semibold transition",
                      selected
                        ? "bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]"
                        : "text-[color:var(--cliply-muted)] hover:bg-white hover:text-[color:var(--cliply-text)]",
                    )}
                  >
                    <Icon className="size-4 shrink-0" />
                    <span>{tab.label}</span>
                  </button>
                );
              })}
            </nav>
          </aside>

          <main className="cliply-scrollbar min-w-0 flex-1 overflow-y-auto bg-[color:var(--cliply-window-bg)] px-6 py-5">
            <div className="mx-auto grid max-w-[720px] gap-4">
              <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white px-4 py-3">
                <h3 className="text-[17px] font-semibold text-[color:var(--cliply-text)]">
                  {activeTabMeta.label}
                </h3>
                <p className="mt-1 text-sm font-medium text-[color:var(--cliply-muted)]">
                  {activeTabMeta.description}
                </p>
              </div>
              {activeContent}
            </div>
          </main>
        </div>

        <footer className="flex h-14 shrink-0 items-center justify-end gap-2 border-t border-[color:var(--cliply-border)] bg-white/90 px-6">
          <button
            type="button"
            onClick={cancelSettings}
            className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-4 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb]"
          >
            取消
          </button>
          <button
            type="button"
            disabled={saveDisabled}
            onClick={() => {
              setCapturingShortcut(false);
              onSave(draft);
            }}
            className="h-9 rounded-lg bg-[color:var(--cliply-accent-strong)] px-4 text-[13px] font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
          >
            保存设置
          </button>
        </footer>
      </section>
    </div>
  );
}

function GeneralSettingsTab({
  draft,
  updateDraft,
}: {
  draft: CliplySettings;
  updateDraft: UpdateSettingsDraft;
}) {
  return (
    <SettingSection icon={Settings2} title="通用行为">
      <ToggleRow
        label="暂停监听"
        checked={draft.pauseMonitoring}
        onChange={(value) => updateDraft("pauseMonitoring", value)}
      />
      <ToggleRow
        label="开机自启"
        checked={draft.launchAtStartup}
        onChange={(value) => updateDraft("launchAtStartup", value)}
      />
      <ToggleRow
        label="启动时最小化到托盘"
        checked={draft.startMinimized}
        onChange={(value) => updateDraft("startMinimized", value)}
      />
      <ToggleRow
        label="打开后自动聚焦搜索框"
        checked={draft.focusSearchOnOpen}
        onChange={(value) => updateDraft("focusSearchOnOpen", value)}
      />
      <ToggleRow
        label="粘贴后自动关闭窗口"
        checked={draft.closeAfterPaste}
        onChange={(value) => updateDraft("closeAfterPaste", value)}
      />
    </SettingSection>
  );
}

function ShortcutsSettingsTab({
  value,
  check,
  capturing,
  onStartCapture,
  onStopCapture,
  onChange,
}: {
  value: string;
  check: ShortcutCheck | null;
  capturing: boolean;
  onStartCapture: () => void;
  onStopCapture: () => void;
  onChange: (value: string) => void;
}) {
  return (
    <div className="grid gap-4">
      <SettingSection icon={Keyboard} title="全局快捷键">
        <ShortcutRecorder
          value={value}
          check={check}
          capturing={capturing}
          onStartCapture={onStartCapture}
          onStopCapture={onStopCapture}
          onChange={onChange}
        />
      </SettingSection>
      <SettingSection icon={Keyboard} title="列表操作">
        <div className="grid gap-2 text-xs font-medium text-[color:var(--cliply-muted)]">
          <ShortcutRow label="粘贴" value="Enter" />
          <ShortcutRow label="无格式粘贴" value="Shift + Enter" />
          <ShortcutRow label="固定" value="Ctrl + P" />
          <ShortcutRow label="删除" value="Delete" />
        </div>
      </SettingSection>
    </div>
  );
}

function PrivacySettingsTab({
  draft,
  ignoreAppsText,
  updateDraft,
}: {
  draft: CliplySettings;
  ignoreAppsText: string;
  updateDraft: UpdateSettingsDraft;
}) {
  return (
    <SettingSection icon={Shield} title="隐私保护">
      <p className="rounded-lg bg-amber-50 px-3 py-2 text-xs leading-5 text-amber-800">
        默认不保存私钥、API token、银行卡号等高风险内容；疑似验证码只保存隐藏占位。
      </p>
      <ToggleRow
        label="启用敏感内容过滤"
        checked={!draft.saveSensitive}
        onChange={(value) => updateDraft("saveSensitive", !value)}
      />
      <ToggleRow
        label="保存图片"
        checked={draft.saveImages}
        onChange={(value) => updateDraft("saveImages", value)}
      />
      <ToggleRow
        label="保存 HTML 富文本"
        checked={draft.saveHtml}
        onChange={(value) => updateDraft("saveHtml", value)}
      />
      <label className="grid gap-2 text-sm font-medium text-[color:var(--cliply-muted)]">
        忽略应用列表
        <textarea
          value={ignoreAppsText}
          onChange={(event) =>
            updateDraft(
              "ignoreApps",
              event.target.value
                .split("\n")
                .map((value) => value.trim())
                .filter(Boolean),
            )
          }
          rows={7}
          className="cliply-scrollbar resize-none rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2 text-sm text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
        />
      </label>
    </SettingSection>
  );
}

function HistorySettingsTab({
  draft,
  updateDraft,
  onClearHistory,
}: {
  draft: CliplySettings;
  updateDraft: UpdateSettingsDraft;
  onClearHistory: () => void;
}) {
  return (
    <SettingSection icon={History} title="历史记录策略">
      <NumberRow
        label="最大历史条数"
        value={draft.maxHistoryItems}
        min={50}
        max={10000}
        onChange={(value) => updateDraft("maxHistoryItems", value)}
      />
      <NumberRow
        label="自动清理天数"
        value={draft.autoDeleteDays}
        min={1}
        max={365}
        onChange={(value) => updateDraft("autoDeleteDays", value)}
      />
      <ToggleRow
        label="忽略重复内容"
        checked={draft.ignoreDuplicate}
        onChange={(value) => updateDraft("ignoreDuplicate", value)}
      />
      <button
        type="button"
        onClick={onClearHistory}
        className="h-10 rounded-xl border border-rose-200 bg-rose-50 px-3 text-left text-sm font-semibold text-rose-700 transition hover:bg-rose-100"
      >
        清空未固定历史
      </button>
    </SettingSection>
  );
}

function AppearanceSettingsTab({
  draft,
  updateDraft,
}: {
  draft: CliplySettings;
  updateDraft: UpdateSettingsDraft;
}) {
  const theme = getCliplyTheme(getDraftThemeName(draft.themeName));
  const accentColor = getAppearanceAccentColor(draft.accentColor, theme.primary);
  const baseColor = theme.primary.toUpperCase();
  const hasCustomAccent = accentColor !== baseColor;
  const previewDescription = hasCustomAccent
    ? `基于${theme.label}方案，使用自定义主题色 ${accentColor}。`
    : theme.description;
  const accentToneWarning = getAccentToneWarning(accentColor);

  return (
    <div className="grid grid-cols-[minmax(0,1fr)_360px] gap-6">
      <div className="grid min-w-0 gap-4">
        <div className="flex h-[78px] items-center justify-between gap-3 rounded-xl border border-[color:var(--cliply-border)] bg-white px-4 py-3 shadow-[0_8px_24px_rgba(15,23,42,0.035)]">
          <div className="min-w-0">
            <div className="text-[13px] font-semibold text-[color:var(--cliply-text)]">
              当前方案：{theme.label}
            </div>
            <div className="mt-1 truncate text-xs font-medium text-[color:var(--cliply-muted)]">
              强调色：{accentColor} · {hasCustomAccent ? "自定义强调色" : "使用方案默认色"}
            </div>
          </div>
          <span
            className="size-9 shrink-0 rounded-xl shadow-sm"
            style={{ backgroundColor: accentColor }}
          />
        </div>

        <SettingSection icon={Sun} title="主题模式">
          <div className="grid grid-cols-3 gap-2">
            <ThemeModeButton
              icon={Sun}
              label="浅色"
              selected={draft.theme === "light"}
              onClick={() => updateDraft("theme", "light")}
            />
            <ThemeModeButton icon={Moon} label="深色" disabled />
            <ThemeModeButton icon={Monitor} label="跟随系统" disabled />
          </div>
        </SettingSection>

        <SettingSection icon={Sparkles} title="主题方案">
          <CompactThemePicker
            value={theme.name}
            onChange={(value) => {
              const nextTheme = getCliplyTheme(value);
              updateDraft("themeName", value);
              updateDraft("accentColor", nextTheme.primary);
            }}
          />
        </SettingSection>

        <SettingSection icon={Palette} title="强调色">
          <div className="grid grid-cols-6 gap-2">
            {ACCENT_PRESET_COLORS.map((color) => {
              const selected = color === accentColor;
              return (
                <button
                  key={color}
                  type="button"
                  aria-label={`使用 ${color}`}
                  title={color}
                  onClick={() => updateDraft("accentColor", color)}
                  className={clsx(
                    "grid size-9 place-items-center rounded-xl border bg-white transition",
                    selected
                      ? "border-[color:var(--cliply-accent)] shadow-[0_0_0_3px_var(--cliply-focus-ring)]"
                      : "border-[color:var(--cliply-border)] hover:border-[color:var(--cliply-border-strong)]",
                  )}
                >
                  <span className="size-7 rounded-full" style={{ backgroundColor: color }} />
                </button>
              );
            })}
          </div>

          <div className="grid grid-cols-[1fr_auto_auto] items-end gap-2">
            <label className="grid gap-1 text-xs font-medium text-[color:var(--cliply-muted)]">
              Hex
              <input
                value={accentColor}
                onChange={(event) => {
                  const nextColor = event.target.value.trim();
                  if (/^#[0-9a-f]{0,6}$/i.test(nextColor)) {
                    updateDraft("accentColor", nextColor.toUpperCase());
                  }
                }}
                onBlur={() => updateDraft("accentColor", accentColor)}
                className="h-10 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)] focus:shadow-[0_0_0_4px_var(--cliply-focus-ring)]"
              />
            </label>
            <label
              className="relative grid size-10 cursor-pointer place-items-center rounded-xl border border-[color:var(--cliply-border)] bg-white transition hover:border-[color:var(--cliply-border-strong)]"
              title="选择强调色"
            >
              <span className="size-7 rounded-full shadow-sm" style={{ backgroundColor: accentColor }} />
              <input
                type="color"
                value={accentColor}
                onChange={(event) => updateDraft("accentColor", event.target.value.toUpperCase())}
                className="absolute inset-0 cursor-pointer opacity-0"
                aria-label="选择强调色"
              />
            </label>
            <button
              type="button"
              onClick={() => updateDraft("accentColor", theme.primary)}
              disabled={!hasCustomAccent}
              className="h-10 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-xs font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled-text)]"
            >
              恢复方案色
            </button>
          </div>

          {accentToneWarning ? (
            <p className="rounded-lg bg-[color:var(--cliply-warning-soft)] px-3 py-2 text-xs font-semibold text-[color:var(--cliply-warning)]">
              {accentToneWarning}
            </p>
          ) : null}
        </SettingSection>
      </div>

      <AppearancePreview
        themeLabel={theme.label}
        accentColor={accentColor}
        description={previewDescription}
        customAccent={hasCustomAccent}
      />
    </div>
  );
}

function ThemeModeButton({
  icon: Icon,
  label,
  selected = false,
  disabled = false,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  selected?: boolean;
  disabled?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={clsx(
        "flex h-10 items-center justify-center gap-2 rounded-lg border text-xs font-semibold transition",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]"
          : "border-[color:var(--cliply-border)] bg-white text-[color:var(--cliply-muted)] hover:border-[color:var(--cliply-border-strong)] hover:text-[color:var(--cliply-text)]",
        disabled && "cursor-not-allowed opacity-45 hover:border-[color:var(--cliply-border)] hover:text-[color:var(--cliply-muted)]",
      )}
    >
      <Icon className="size-4" />
      {label}
    </button>
  );
}

function CompactThemePicker({
  value,
  onChange,
}: {
  value: CliplyThemeName;
  onChange: (value: CliplyThemeName) => void;
}) {
  return (
    <div className="grid grid-cols-2 gap-2">
      {CLIPLY_THEME_OPTIONS.map((theme) => {
        const selected = theme.name === value;
        return (
          <button
            key={theme.name}
            type="button"
            onClick={() => onChange(theme.name)}
            className={clsx(
              "flex h-[72px] items-center gap-3 rounded-xl border bg-white px-3 text-left transition",
              selected
                ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] shadow-[0_0_0_3px_var(--cliply-focus-ring)]"
                : "border-[color:var(--cliply-border)] hover:border-[color:var(--cliply-border-strong)] hover:bg-[#fbfcfe]",
            )}
          >
            <span className="size-7 shrink-0 rounded-full" style={{ backgroundColor: theme.swatch }} />
            <span className="min-w-0 flex-1">
              <span className="block truncate text-[13px] font-semibold text-[color:var(--cliply-text)]">
                {theme.label}
              </span>
              <span className="mt-1 block truncate text-xs font-medium text-[color:var(--cliply-muted)]">
                {THEME_SUMMARIES[theme.name]}
              </span>
            </span>
            {selected ? (
              <span className="grid size-5 shrink-0 place-items-center rounded-full bg-[color:var(--cliply-accent)] text-[color:var(--cliply-primary-text)]">
                <Check className="size-3.5" />
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}

function AppearancePreview({
  themeLabel,
  accentColor,
  description,
  customAccent,
}: {
  themeLabel: string;
  accentColor: string;
  description: string;
  customAccent: boolean;
}) {
  return (
    <aside className="sticky top-6 self-start">
      <div className="overflow-hidden rounded-2xl border border-[color:var(--cliply-border)] bg-white shadow-[0_16px_40px_rgba(15,23,42,0.08)]">
        <div className="flex items-center justify-between gap-3 border-b border-[color:var(--cliply-border-soft)] px-4 py-3">
          <div className="min-w-0">
            <div className="text-[13px] font-semibold text-[color:var(--cliply-text)]">
              实时界面预览
            </div>
            <div className="mt-1 truncate text-xs font-medium text-[color:var(--cliply-muted)]">
              {themeLabel} · {customAccent ? "自定义强调色" : "方案默认色"}
            </div>
          </div>
          <Badge tone="accent">{accentColor}</Badge>
        </div>

        <div className="bg-[color:var(--cliply-window-bg)] p-4">
          <div className="overflow-hidden rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-bg)] shadow-[var(--cliply-shadow-panel)]">
            <div className="flex h-10 items-center justify-between px-3">
              <div className="flex items-center gap-2">
                <span
                  className="grid size-6 place-items-center rounded-lg text-[11px] font-bold text-[color:var(--cliply-primary-text)]"
                  style={{ backgroundColor: accentColor }}
                >
                  C
                </span>
                <span className="text-[13px] font-semibold text-[color:var(--cliply-text)]">Cliply</span>
              </div>
              <span className="text-[16px] leading-none text-[color:var(--cliply-muted)]">...</span>
            </div>

            <div className="px-3 pb-3">
              <div className="flex h-9 items-center gap-2 rounded-xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-input-bg)] px-3 shadow-[0_0_0_4px_var(--cliply-focus-ring)]">
                <span className="size-2 rounded-full bg-[color:var(--cliply-accent)]" />
                <span className="text-xs font-medium text-[color:var(--cliply-placeholder)]">
                  搜索剪贴板...
                </span>
              </div>

              <div className="mt-3 flex gap-2">
                <span className="rounded-full bg-[color:var(--cliply-accent-50)] px-3 py-1 text-xs font-semibold text-[color:var(--cliply-accent-strong)]">
                  全部
                </span>
                <span className="rounded-full bg-[color:var(--cliply-muted-bg)] px-3 py-1 text-xs font-semibold text-[color:var(--cliply-muted)]">
                  图片
                </span>
                <span className="rounded-full bg-[color:var(--cliply-muted-bg)] px-3 py-1 text-xs font-semibold text-[color:var(--cliply-muted)]">
                  固定
                </span>
              </div>

              <div className="mt-3 rounded-xl border border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] p-3 shadow-[var(--cliply-shadow-selected)]">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-xs font-semibold text-[color:var(--cliply-muted)]">
                      文本 · Cliply
                    </div>
                    <div className="mt-1 truncate text-[13px] font-semibold text-[color:var(--cliply-text)]">
                      选中剪贴板记录
                    </div>
                    <div className="mt-1 text-xs font-medium text-[color:var(--cliply-muted)]">
                      badge、边框、焦点态同步变色
                    </div>
                  </div>
                  <span
                    className="grid size-8 shrink-0 place-items-center rounded-lg text-xs font-semibold text-[color:var(--cliply-primary-text)]"
                    style={{ backgroundColor: accentColor }}
                  >
                    ✓
                  </span>
                </div>
              </div>

              <div className="mt-3 flex items-center justify-between gap-2">
                <button
                  type="button"
                  className="h-9 rounded-xl px-4 text-xs font-semibold"
                  style={{
                    backgroundColor: accentColor,
                    color: "var(--cliply-primary-text)",
                  }}
                >
                  粘贴
                </button>
                <button
                  type="button"
                  className="h-9 rounded-xl border border-[color:var(--cliply-border)] bg-white px-4 text-xs font-semibold text-[color:var(--cliply-text)]"
                >
                  复制
                </button>
                <span className="rounded-lg bg-[color:var(--cliply-accent-50)] px-2 py-1 text-[11px] font-semibold text-[color:var(--cliply-accent-strong)]">
                  badge
                </span>
              </div>
            </div>
          </div>

          <p className="mt-3 text-xs font-medium leading-5 text-[color:var(--cliply-muted)]">
            {description}
          </p>
        </div>
      </div>
    </aside>
  );
}

function SyncSettingsTab({
  remoteSyncStatus,
  syncStatus,
  selectedSyncProviderType,
  webdavDraft,
  ftpDraft,
  syncPassword,
  autoSyncEnabled,
  autoSyncIntervalMinutes,
  syncBusy,
  providerBusy,
  syncMessage,
  syncError,
  syncActionDisabled,
  remoteSyncActionDisabled,
  setWebdavDraft,
  setFtpDraft,
  setSyncPassword,
  setAutoSyncEnabled,
  setAutoSyncIntervalMinutes,
  onProviderChange,
  onChooseSyncFolder,
  onSaveWebdavProvider,
  onSaveFtpProvider,
  onSaveAutoSync,
  onClearAutoSyncPassword,
  onSyncNow,
  onExportSyncPackage,
  onImportSyncPackage,
  onExportToRemote,
  onImportFromRemote,
}: {
  remoteSyncStatus: RemoteSyncStatus;
  syncStatus: SyncPackageStatus;
  selectedSyncProviderType: SyncProviderConfig["type"];
  webdavDraft: WebdavProviderConfig;
  ftpDraft: FtpProviderConfig;
  syncPassword: string;
  autoSyncEnabled: boolean;
  autoSyncIntervalMinutes: number;
  syncBusy: "export" | "import" | "sync" | null;
  providerBusy: boolean;
  syncMessage: string | null;
  syncError: string | null;
  syncActionDisabled: boolean;
  remoteSyncActionDisabled: boolean;
  setWebdavDraft: Dispatch<SetStateAction<WebdavProviderConfig>>;
  setFtpDraft: Dispatch<SetStateAction<FtpProviderConfig>>;
  setSyncPassword: (value: string) => void;
  setAutoSyncEnabled: (value: boolean) => void;
  setAutoSyncIntervalMinutes: (value: number) => void;
  onProviderChange: (type: SyncProviderConfig["type"]) => void | Promise<void>;
  onChooseSyncFolder: () => void | Promise<void>;
  onSaveWebdavProvider: () => void | Promise<void>;
  onSaveFtpProvider: () => void | Promise<void>;
  onSaveAutoSync: () => void | Promise<void>;
  onClearAutoSyncPassword: () => void | Promise<void>;
  onSyncNow: () => void | Promise<void>;
  onExportSyncPackage: () => void | Promise<void>;
  onImportSyncPackage: () => void | Promise<void>;
  onExportToRemote: () => void | Promise<void>;
  onImportFromRemote: () => void | Promise<void>;
}) {
  return (
    <div className="grid gap-4">
      <SettingSection icon={RefreshCw} title="同步状态">
        <p className="rounded-lg bg-[color:var(--cliply-accent-50)] px-3 py-2 text-xs leading-5 text-[color:var(--cliply-text-secondary)]">
          同步包已加密，请妥善保存同步密码。Cliply 不会把明文剪贴板内容写入远程目录。
        </p>
        <div className="grid grid-cols-3 gap-2 text-xs font-medium text-[color:var(--cliply-muted)]">
          <SyncStat label="Manifest" value={remoteSyncStatus.manifestExists ? "已检测" : "未检测"} />
          <SyncStat label="快照" value={String(remoteSyncStatus.snapshotCount)} />
          <SyncStat label="状态" value={remoteSyncStatus.lastStatus || "暂无"} />
        </div>
      </SettingSection>

      <SettingSection icon={RefreshCw} title="同步方式">
        <div className="grid grid-cols-4 gap-2">
          {SYNC_PROVIDER_OPTIONS.map((option) => {
            const selected = selectedSyncProviderType === option.type;
            return (
              <button
                key={option.type}
                type="button"
                onClick={() => void onProviderChange(option.type)}
                className={clsx(
                  "flex h-10 items-center justify-center rounded-lg border px-2 text-center text-[13px] font-semibold transition",
                  selected
                    ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]"
                    : "border-[color:var(--cliply-border)] bg-white text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)]",
                )}
              >
                {option.label}
              </button>
            );
          })}
        </div>
      </SettingSection>

      {selectedSyncProviderType === "local-folder" ? (
        <SettingSection icon={RefreshCw} title="本地文件夹配置">
          <div className="flex items-center justify-between gap-3 rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2">
            <div className="min-w-0">
              <div className="text-xs font-medium text-[color:var(--cliply-muted)]">同步文件夹</div>
              <div className="cliply-code-font mt-1 truncate text-[13px] font-semibold text-[color:var(--cliply-text)]">
                {remoteSyncStatus.provider.type === "local-folder"
                  ? remoteSyncStatus.provider.path
                  : "尚未选择"}
              </div>
            </div>
            <button
              type="button"
              disabled={providerBusy}
              onClick={() => void onChooseSyncFolder()}
              className="h-8 shrink-0 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-xs font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
            >
              选择文件夹
            </button>
          </div>
        </SettingSection>
      ) : null}

      {selectedSyncProviderType === "webdav" ? (
        <SettingSection icon={RefreshCw} title="WebDAV 配置">
          <TextInput
            label="WebDAV 地址"
            value={webdavDraft.url}
            placeholder="https://example.com/remote.php/dav/files/user/"
            onChange={(value) => setWebdavDraft((current) => ({ ...current, url: value }))}
          />
          <div className="grid grid-cols-2 gap-3">
            <TextInput
              label="用户名"
              value={webdavDraft.username}
              placeholder="webdav user"
              onChange={(value) => setWebdavDraft((current) => ({ ...current, username: value }))}
            />
            <TextInput
              label="密码"
              type="password"
              value={webdavDraft.password}
              placeholder="webdav password"
              onChange={(value) => setWebdavDraft((current) => ({ ...current, password: value }))}
            />
          </div>
          <div className="grid grid-cols-[1fr_auto] items-end gap-3">
            <TextInput
              label="远程目录"
              value={webdavDraft.remotePath}
              placeholder="cliply"
              onChange={(value) =>
                setWebdavDraft((current) => ({ ...current, remotePath: value }))
              }
            />
            <button
              type="button"
              disabled={providerBusy}
              onClick={() => void onSaveWebdavProvider()}
              className="h-8 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-xs font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
            >
              {providerBusy ? "保存中..." : "保存 WebDAV"}
            </button>
          </div>
        </SettingSection>
      ) : null}

      {selectedSyncProviderType === "ftp" ? (
        <SettingSection icon={RefreshCw} title="FTP/FTPS 配置">
          <div className="grid grid-cols-[1fr_96px] gap-3">
            <TextInput
              label="主机"
              value={ftpDraft.host}
              placeholder="example.com"
              onChange={(value) => setFtpDraft((current) => ({ ...current, host: value }))}
            />
            <TextInput
              label="端口"
              value={String(ftpDraft.port || 21)}
              placeholder="21"
              onChange={(value) =>
                setFtpDraft((current) => ({ ...current, port: normalizePort(value) }))
              }
            />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <TextInput
              label="用户名"
              value={ftpDraft.username}
              placeholder="ftp user"
              onChange={(value) => setFtpDraft((current) => ({ ...current, username: value }))}
            />
            <TextInput
              label="密码"
              type="password"
              value={ftpDraft.password}
              placeholder="ftp password"
              onChange={(value) => setFtpDraft((current) => ({ ...current, password: value }))}
            />
          </div>
          <div className="grid grid-cols-[1fr_auto] items-end gap-3">
            <TextInput
              label="远程目录"
              value={ftpDraft.remotePath}
              placeholder="/mnt/user/sync"
              onChange={(value) =>
                setFtpDraft((current) => ({ ...current, remotePath: value }))
              }
              onBlur={() =>
                setFtpDraft((current) => ({
                  ...current,
                  remotePath: normalizeRemotePath(current.remotePath),
                }))
              }
            />
            <button
              type="button"
              disabled={providerBusy}
              onClick={() => void onSaveFtpProvider()}
              className="h-8 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-xs font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
            >
              {providerBusy ? "保存中..." : "保存 FTP"}
            </button>
          </div>
          <ToggleRow
            label="使用 FTPS"
            checked={ftpDraft.secure}
            onChange={(value) => setFtpDraft((current) => ({ ...current, secure: value }))}
          />
        </SettingSection>
      ) : null}

      {selectedSyncProviderType !== "disabled" ? (
        <SettingSection icon={RefreshCw} title="自动同步">
          <div className="flex items-center justify-between gap-3">
            <ToggleRow label="启用自动同步" checked={autoSyncEnabled} onChange={setAutoSyncEnabled} />
            <Badge tone={remoteSyncStatus.syncPasswordSaved ? "teal" : "neutral"}>
              {remoteSyncStatus.syncPasswordSaved ? "已保存密码" : "未保存密码"}
            </Badge>
          </div>
          <NumberRow
            label="同步间隔（分钟）"
            value={autoSyncIntervalMinutes}
            min={1}
            max={1440}
            onChange={setAutoSyncIntervalMinutes}
          />
          <div className="flex min-w-0 items-center justify-between gap-3 rounded-lg bg-[#fafafb] px-2.5 py-1.5 text-[12px] font-medium text-[color:var(--cliply-muted)]">
            <span>自动同步：{remoteSyncStatus.autoSyncEnabled ? "已开启" : "已关闭"}</span>
            <span className="truncate">最近自动：{formatSyncTime(remoteSyncStatus.lastAutoSyncAt)}</span>
          </div>
        </SettingSection>
      ) : null}

      <SettingSection icon={Shield} title="加密">
        <div className="flex items-center justify-between gap-3 rounded-lg bg-[#fafafb] px-3 py-2">
          <span className="text-xs font-medium text-[color:var(--cliply-muted)]">
            同一个密码用于同步包加密和自动同步。
          </span>
          <Badge tone={remoteSyncStatus.syncPasswordSaved ? "teal" : "neutral"}>
            {remoteSyncStatus.syncPasswordSaved ? "已保存密码" : "未保存密码"}
          </Badge>
        </div>
        <label className="grid gap-1.5 text-sm font-medium text-[color:var(--cliply-muted)]">
          同步密码
          <input
            type="password"
            value={syncPassword}
            onChange={(event) => setSyncPassword(event.target.value)}
            placeholder={
              remoteSyncStatus.syncPasswordSaved
                ? "留空则继续使用已保存密码"
                : "用于加密 .cliply-sync 文件，也可保存给自动同步"
            }
            className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
          />
        </label>
        <div className="grid grid-cols-2 gap-2">
          <button
            type="button"
            disabled={providerBusy}
            onClick={() => void onSaveAutoSync()}
            className="h-9 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-[13px] font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
          >
            {providerBusy ? "保存中..." : "保存同步配置"}
          </button>
          <button
            type="button"
            disabled={providerBusy || !remoteSyncStatus.syncPasswordSaved}
            onClick={() => void onClearAutoSyncPassword()}
            className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:border-[color:var(--cliply-border-strong)] hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:bg-[#f5f7fb] disabled:text-[color:var(--cliply-disabled-text)]"
          >
            清除已保存密码
          </button>
        </div>
      </SettingSection>

      <SettingSection icon={RefreshCw} title="手动操作">
        <div className="grid grid-cols-3 gap-2">
          <button
            type="button"
            disabled={
              remoteSyncActionDisabled ||
              selectedSyncProviderType !== remoteSyncStatus.provider.type ||
              !canUseRemoteProvider(remoteSyncStatus.provider)
            }
            onClick={() => void onSyncNow()}
            className="h-9 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-[13px] font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
          >
            {syncBusy === "sync" ? "同步中..." : "立即同步"}
          </button>
          <button
            type="button"
            disabled={syncActionDisabled}
            onClick={() => void onExportSyncPackage()}
            className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled-text)]"
          >
            {syncBusy === "export" ? "导出中..." : "导出同步包"}
          </button>
          <button
            type="button"
            disabled={syncActionDisabled}
            onClick={() => void onImportSyncPackage()}
            className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled-text)]"
          >
            {syncBusy === "import" ? "导入中..." : "导入同步包"}
          </button>
        </div>
        {selectedSyncProviderType !== "disabled" ? (
          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              disabled={
                syncActionDisabled ||
                selectedSyncProviderType !== remoteSyncStatus.provider.type ||
                !canUseRemoteProvider(remoteSyncStatus.provider)
              }
              onClick={() => void onExportToRemote()}
              className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled-text)]"
            >
              {syncBusy === "export"
                ? "导出中..."
                : remoteSyncActionLabel(selectedSyncProviderType, "export")}
            </button>
            <button
              type="button"
              disabled={
                syncActionDisabled ||
                selectedSyncProviderType !== remoteSyncStatus.provider.type ||
                !canUseRemoteProvider(remoteSyncStatus.provider)
              }
              onClick={() => void onImportFromRemote()}
              className="h-9 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb] disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled-text)]"
            >
              {syncBusy === "import"
                ? "导入中..."
                : remoteSyncActionLabel(selectedSyncProviderType, "import")}
            </button>
          </div>
        ) : null}
      </SettingSection>

      <SettingSection icon={History} title="最近同步状态">
        <div className="grid gap-1 rounded-lg bg-[#fafafb] px-3 py-2 text-xs font-medium text-[color:var(--cliply-muted)]">
          <span>最近导出：{formatSyncTime(syncStatus.lastExportedAt)}</span>
          <span>最近导入：{formatSyncTime(syncStatus.lastImportedAt)}</span>
          <span>最近同步：{formatSyncTime(remoteSyncStatus.lastSyncedAt)}</span>
        </div>
        {syncMessage ? (
          <p className="rounded-lg bg-emerald-50 px-3 py-2 text-xs font-semibold text-emerald-700">
            {syncMessage}
          </p>
        ) : null}
        {syncError ? (
          <p className="rounded-lg bg-rose-50 px-3 py-2 text-xs font-semibold text-rose-700">
            {syncError}
          </p>
        ) : null}
      </SettingSection>
    </div>
  );
}

function AboutSettingsTab({ debugInfo }: { debugInfo: CliplyDebugInfo | null }) {
  return (
    <div className="grid gap-4">
      <SettingSection icon={CircleHelp} title="Cliply">
        <div className="flex items-center justify-between rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2">
          <div>
            <div className="text-[15px] font-semibold text-[color:var(--cliply-text)]">Cliply</div>
            <div className="mt-1 text-xs font-medium text-[color:var(--cliply-muted)]">
              Local-first clipboard manager
            </div>
          </div>
          <Badge tone="accent">v{CLIPLY_VERSION}</Badge>
        </div>
        <DebugPathRow label="数据目录" value={debugInfo?.dataDir} />
        <DebugPathRow label="日志目录" value={directoryOf(debugInfo?.logPath)} />
        <DebugPathRow label="数据库文件" value={debugInfo?.databasePath} />
      </SettingSection>
      <SettingSection icon={RefreshCw} title="更新">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2">
          <div className="text-sm font-medium text-[color:var(--cliply-muted)]">
            检查更新将在后续版本接入。
          </div>
          <button
            type="button"
            disabled
            className="h-8 rounded-lg border border-[color:var(--cliply-border)] bg-[#f5f7fb] px-3 text-xs font-semibold text-[color:var(--cliply-disabled-text)]"
          >
            检查更新
          </button>
        </div>
      </SettingSection>
    </div>
  );
}

function SyncStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg bg-[#fafafb] px-3 py-2">
      <div className="text-[11px] font-semibold text-[color:var(--cliply-muted)]">{label}</div>
      <div className="mt-1 truncate text-[13px] font-semibold text-[color:var(--cliply-text)]">
        {value}
      </div>
    </div>
  );
}

function DebugPathRow({ label, value }: { label: string; value?: string | null }) {
  return (
    <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2">
      <div className="text-xs font-medium text-[color:var(--cliply-muted)]">{label}</div>
      <div className="cliply-code-font mt-1 cursor-text select-text break-all text-[12px] font-semibold text-[color:var(--cliply-text)]">
        {value || "正在读取..."}
      </div>
    </div>
  );
}

type SectionIcon = LucideIcon;

function SettingSection({
  icon: Icon,
  title,
  children,
}: {
  icon: SectionIcon;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-xl border border-[color:var(--cliply-border)] bg-white p-4 shadow-[0_8px_24px_rgba(15,23,42,0.035)]">
      <div className="mb-3 flex items-center gap-2 text-[13px] font-semibold text-[color:var(--cliply-text)]">
        <Icon className="size-4 text-[color:var(--cliply-accent)]" />
        {title}
      </div>
      <div className="grid gap-2.5">{children}</div>
    </section>
  );
}

function ShortcutRecorder({
  value,
  check,
  capturing,
  onStartCapture,
  onStopCapture,
  onChange,
}: {
  value: string;
  check: ShortcutCheck | null;
  capturing: boolean;
  onStartCapture: () => void;
  onStopCapture: () => void;
  onChange: (value: string) => void;
}) {
  const statusTone = check?.ok ? "ok" : check ? "error" : "pending";
  const statusLabel = capturing
    ? "录制中"
    : statusTone === "ok"
      ? "可用"
      : statusTone === "error"
        ? "不可用"
        : "检测中";

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (!capturing) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        event.stopPropagation();
        onStartCapture();
      }
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      onStopCapture();
      return;
    }

    if (event.key === "Backspace" || event.key === "Delete") {
      onChange("");
      onStopCapture();
      return;
    }

    const shortcut = shortcutFromKeyboardEvent(event);
    if (shortcut === null) {
      return;
    }

    onChange(shortcut);
    onStopCapture();
  };

  return (
    <div className="grid gap-2 text-sm font-medium text-[color:var(--cliply-muted)]">
      <div className="flex items-center justify-between gap-3">
        <span>打开 Cliply</span>
        <span
          className={clsx(
            "rounded-full px-2 py-0.5 text-[11px] font-semibold",
            capturing && "bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]",
            !capturing && statusTone === "ok" && "bg-emerald-50 text-emerald-700",
            !capturing && statusTone === "error" && "bg-rose-50 text-rose-700",
            !capturing && statusTone === "pending" && "bg-[#f3f5f8] text-[color:var(--cliply-muted)]",
          )}
        >
          {statusLabel}
        </span>
      </div>
      <button
        type="button"
        onClick={onStartCapture}
        onBlur={onStopCapture}
        onKeyDown={handleKeyDown}
        className={clsx(
          "flex h-11 items-center justify-between rounded-xl border bg-white px-3 text-left text-sm font-semibold outline-none transition",
          capturing
            ? "border-[color:var(--cliply-accent)] text-[color:var(--cliply-accent-strong)] shadow-[0_0_0_4px_var(--cliply-focus-ring)]"
            : check?.ok
              ? "border-emerald-200 text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)]"
              : check
                ? "border-rose-200 text-[color:var(--cliply-text)] hover:border-rose-300"
                : "border-[color:var(--cliply-border)] text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)]",
        )}
      >
        <span className="cliply-code-font truncate">
          {capturing ? "按下新的快捷键..." : value || "点击录制快捷键"}
        </span>
        <span className="ml-3 shrink-0 text-xs font-medium text-[color:var(--cliply-muted)]">
          {capturing ? "Esc 取消" : "点击修改"}
        </span>
      </button>
      <p
        className={clsx(
          "min-h-5 text-xs leading-5",
          check?.ok ? "text-emerald-700" : check ? "text-rose-700" : "text-[color:var(--cliply-muted)]",
        )}
      >
        {capturing
          ? "需要包含 Ctrl、Alt 或 Win；按 Backspace/Delete 可清空。"
          : check?.message ?? "正在检测快捷键是否可用..."}
      </p>
    </div>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex h-8 items-center justify-between gap-3 text-[13px] font-medium text-[color:var(--cliply-muted)]">
      {label}
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        className="size-4 accent-[color:var(--cliply-accent)]"
      />
    </label>
  );
}

function TextInput({
  label,
  value,
  placeholder,
  type = "text",
  onChange,
  onBlur,
}: {
  label: string;
  value: string;
  placeholder?: string;
  type?: "text" | "password";
  onChange: (value: string) => void;
  onBlur?: () => void;
}) {
  return (
    <label className="grid min-w-0 gap-1 text-xs font-medium text-[color:var(--cliply-muted)]">
      {label}
      <input
        type={type}
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        onBlur={onBlur}
        className="h-8 w-full min-w-0 rounded-lg border border-[color:var(--cliply-border)] bg-white px-2.5 text-[13px] font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
      />
    </label>
  );
}

function NumberRow({
  label,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-3 text-sm font-medium text-[color:var(--cliply-muted)]">
      {label}
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="h-8 w-24 rounded-lg border border-[color:var(--cliply-border)] bg-white px-2.5 text-right text-[13px] font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
      />
    </label>
  );
}

function ShortcutRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-2 rounded-lg bg-[#fafafb] px-3 py-2">
      <span>{label}</span>
      <span className="cliply-code-font text-[color:var(--cliply-text)]">{value}</span>
    </div>
  );
}

function getDraftThemeName(value: string): CliplyThemeName {
  return isCliplyThemeName(value) ? value : DEFAULT_THEME_NAME;
}

function getAppearanceAccentColor(value: string | undefined, fallback: string) {
  const color = value?.trim();
  return /^#[0-9a-f]{6}$/i.test(color ?? "") ? color!.toUpperCase() : fallback.toUpperCase();
}

function getAccentToneWarning(hex: string) {
  const rgb = hexToRgb(hex);
  if (!rgb) {
    return null;
  }

  const luminance = (0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b) / 255;
  if (luminance < 0.16 || luminance > 0.86) {
    return "该颜色在浅色主题下可能显得过重。";
  }

  return null;
}

function hexToRgb(hex: string) {
  const normalized = /^#[0-9a-f]{6}$/i.test(hex) ? hex : null;
  if (!normalized) {
    return null;
  }

  return {
    r: Number.parseInt(normalized.slice(1, 3), 16),
    g: Number.parseInt(normalized.slice(3, 5), 16),
    b: Number.parseInt(normalized.slice(5, 7), 16),
  };
}

function defaultWebdavConfig(): WebdavProviderConfig {
  return {
    type: "webdav",
    url: "",
    username: "",
    password: "",
    remotePath: "cliply",
  };
}

function normalizeWebdavConfig(config: WebdavProviderConfig): WebdavProviderConfig {
  return {
    ...config,
    url: config.url.trim(),
    username: config.username.trim(),
    remotePath: normalizeWebdavRemotePath(config.remotePath),
  };
}

function hasWebdavDraft(config: WebdavProviderConfig) {
  return Boolean(config.url.trim() || config.username.trim() || config.password || config.remotePath.trim());
}

function defaultFtpConfig(): FtpProviderConfig {
  return {
    type: "ftp",
    host: "",
    port: 21,
    username: "",
    password: "",
    secure: false,
    remotePath: "cliply",
  };
}

function normalizeFtpConfig(config: FtpProviderConfig): FtpProviderConfig {
  return {
    ...config,
    host: config.host.trim(),
    port: config.port || (config.secure ? 21 : 21),
    username: config.username.trim(),
    remotePath: normalizeRemotePath(config.remotePath),
  };
}

function hasFtpDraft(config: FtpProviderConfig) {
  return Boolean(config.host.trim() || config.username.trim() || config.password || config.remotePath.trim());
}

function normalizeWebdavRemotePath(value: string) {
  return value
    .replace(/\\/g, "/")
    .trim()
    .split("/")
    .map((part) => part.trim())
    .filter((part) => part && part !== "." && part !== "..")
    .join("/");
}

function normalizeRemotePath(value: string) {
  const normalized = value.replace(/\\/g, "/").trim();
  const isAbsolute = normalized.startsWith("/");
  const path = normalized
    .split("/")
    .map((part) => part.trim())
    .filter((part) => part && part !== "." && part !== "..")
    .join("/");
  if (path.startsWith("mnt/")) {
    return `/${path}`;
  }
  return isAbsolute && path ? `/${path}` : path;
}

function normalizePort(value: string) {
  const port = Number.parseInt(value, 10);
  if (!Number.isFinite(port)) {
    return 21;
  }
  return Math.min(Math.max(port, 1), 65535);
}

function canUseRemoteProvider(provider: SyncProviderConfig) {
  if (provider.type === "local-folder") {
    return Boolean(provider.path.trim());
  }
  if (provider.type === "webdav") {
    return Boolean(provider.url.trim() && provider.username.trim() && provider.password);
  }
  if (provider.type === "ftp") {
    return Boolean(provider.host.trim() && provider.username.trim() && provider.password);
  }
  return false;
}

function remoteSyncActionLabel(type: SyncProviderConfig["type"], action: "export" | "import") {
  if (type === "webdav") {
    return action === "export" ? "导出到 WebDAV" : "从 WebDAV 导入";
  }
  if (type === "ftp") {
    return action === "export" ? "导出到 FTP/FTPS" : "从 FTP/FTPS 导入";
  }
  return action === "export" ? "导出到同步文件夹" : "从同步文件夹导入";
}

function remoteSyncProviderLabel(type: SyncProviderConfig["type"]) {
  if (type === "webdav") {
    return "WebDAV";
  }
  if (type === "ftp") {
    return "FTP/FTPS";
  }
  return "远程同步";
}

function directoryOf(path?: string | null) {
  if (!path) {
    return null;
  }

  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  if (index <= 0) {
    return path;
  }

  return normalized.slice(0, index);
}

function shortcutFromKeyboardEvent(event: ReactKeyboardEvent<HTMLButtonElement>) {
  if (event.key === "Tab") {
    return null;
  }

  const key = normalizeShortcutKey(event.key, event.code);
  if (!key) {
    return null;
  }

  return [
    event.ctrlKey ? "Ctrl" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
    event.metaKey ? "Win" : null,
    key,
  ]
    .filter(Boolean)
    .join("+");
}

function normalizeShortcutKey(key: string, code: string) {
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) {
    return null;
  }

  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }

  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key)) {
    return key;
  }

  if (key.length === 1) {
    return key.toUpperCase();
  }

  const knownKeys: Record<string, string> = {
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    ArrowUp: "ArrowUp",
    Backspace: "Backspace",
    Delete: "Delete",
    End: "End",
    Enter: "Enter",
    Escape: "Escape",
    Home: "Home",
    Insert: "Insert",
    PageDown: "PageDown",
    PageUp: "PageUp",
    Space: "Space",
    Tab: "Tab",
  };

  return knownKeys[key] ?? null;
}

function formatSyncTime(value?: string | null) {
  if (!value) {
    return "暂无";
  }

  try {
    return new Intl.DateTimeFormat("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(value));
  } catch {
    return value;
  }
}

function syncImportResultMessage(result: SyncImportResult) {
  return `导入完成：新增 ${result.importedCount}，更新 ${result.updatedCount}，删除 ${result.deletedCount}，跳过 ${result.skippedCount}，冲突 ${result.conflictedCount}`;
}

function remoteSyncResultMessage(result: RemoteSyncResult, prefix: string) {
  return `${prefix}：快照 ${result.snapshotCount}，新增 ${result.importedCount}，更新 ${result.updatedCount}，删除 ${result.deletedCount}，冲突 ${result.conflictedCount}`;
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
