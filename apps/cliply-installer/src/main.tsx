import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  ArrowRight,
  Check,
  ChevronRight,
  CircleCheck,
  FolderOpen,
  HardDrive,
  History,
  Loader2,
  LockKeyhole,
  Minus,
  Monitor,
  Power,
  RefreshCw,
  ShieldCheck,
  Trash2,
  X,
} from "lucide-react";
import "./styles.css";

type InstallDetection = {
  isUpdate: boolean;
  installDir: string;
  existingInstallDir?: string | null;
  defaultInstallDir: string;
};

type InstallerMode = {
  isUninstall: boolean;
  isUpdate: boolean;
  installDir?: string | null;
  sourceVersion?: string | null;
  targetVersion?: string | null;
  preserveUserData: boolean;
  launchAfterInstall: boolean;
  parentPid?: number | null;
};

type InstallProgress = {
  progress: number;
  step: string;
};

type InstallOutcome = {
  installDir: string;
  isUpdate: boolean;
};

type UninstallOutcome = {
  installDir: string;
  userDataRemoved: boolean;
};

type Screen = "setup" | "working" | "complete";

const DEFAULT_DETECTION: InstallDetection = {
  isUpdate: false,
  installDir: "C:\\Program Files\\Cliply",
  existingInstallDir: null,
  defaultInstallDir: "C:\\Program Files\\Cliply",
};

function normalizeError(reason: unknown) {
  const message = reason instanceof Error ? reason.message : String(reason);
  return message.replace(/^Error:\s*/i, "");
}

function App() {
  const [screen, setScreen] = React.useState<Screen>("setup");
  const [mode, setMode] = React.useState<InstallerMode>({
    isUninstall: false,
    isUpdate: false,
    installDir: null,
    sourceVersion: null,
    targetVersion: null,
    preserveUserData: false,
    launchAfterInstall: false,
    parentPid: null,
  });
  const [detection, setDetection] =
    React.useState<InstallDetection>(DEFAULT_DETECTION);
  const [installDir, setInstallDir] = React.useState(DEFAULT_DETECTION.installDir);
  const [desktopShortcut, setDesktopShortcut] = React.useState(true);
  const [startOnLogin, setStartOnLogin] = React.useState(true);
  const [launchAfterInstall, setLaunchAfterInstall] = React.useState(true);
  const [removeUserData, setRemoveUserData] = React.useState(false);
  const [progress, setProgress] = React.useState<InstallProgress>({
    progress: 0,
    step: "正在准备安装",
  });
  const [error, setError] = React.useState<string | null>(null);
  const [installOutcome, setInstallOutcome] =
    React.useState<InstallOutcome | null>(null);
  const [uninstallOutcome, setUninstallOutcome] =
    React.useState<UninstallOutcome | null>(null);
  const modeInstallDirRef = React.useRef<string | null>(null);

  React.useEffect(() => {
    void invoke<InstallerMode>("detect_mode")
      .then((nextMode) => {
        setMode(nextMode);
        if (nextMode.installDir) {
          modeInstallDirRef.current = nextMode.installDir;
          setInstallDir(nextMode.installDir);
        }
      })
      .catch((reason) => setError(normalizeError(reason)));

    void invoke<InstallDetection>("detect_installation")
      .then((nextDetection) => {
        setDetection(nextDetection);
        if (!modeInstallDirRef.current) {
          setInstallDir(nextDetection.installDir);
        }
      })
      .catch((reason) => setError(normalizeError(reason)));

    const unlisten = listen<InstallProgress>("installer-progress", (event) => {
      setProgress(event.payload);
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  const isUpdate = mode.isUpdate || detection.isUpdate;
  const isUninstall = mode.isUninstall;

  React.useEffect(() => {
    if (!mode.isUpdate || screen !== "setup") {
      return;
    }
    void install();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode.isUpdate]);

  async function chooseInstallDir() {
    setError(null);
    try {
      const selected = await invoke<string | null>("browse_install_dir", {
        currentDir: installDir,
      });
      if (selected) {
        setInstallDir(selected);
      }
    } catch (reason) {
      setError(normalizeError(reason));
    }
  }

  async function install() {
    setError(null);
    setProgress({
      progress: 0,
      step: isUpdate ? "正在准备更新 Cliply" : "正在准备安装 Cliply",
    });
    setScreen("working");

    try {
      const updateMode = mode.isUpdate;
      const nextOutcome = await invoke<InstallOutcome>("run_install", {
        options: {
          installDir,
          createDesktopShortcut: updateMode ? false : desktopShortcut,
          startOnLogin: updateMode ? false : startOnLogin,
          isUpdate,
          preserveUserData: mode.preserveUserData || isUpdate,
          launchAfterInstall: false,
          parentPid: mode.parentPid ?? null,
        },
      });
      setInstallOutcome(nextOutcome);
      setProgress({ progress: 100, step: "安装完成" });
      setScreen("complete");
    } catch (reason) {
      setError(normalizeError(reason));
      setScreen(mode.isUpdate ? "working" : "setup");
    }
  }

  async function uninstall() {
    setError(null);
    setProgress({ progress: 0, step: "正在准备卸载 Cliply" });
    setScreen("working");

    try {
      const nextOutcome = await invoke<UninstallOutcome>("run_uninstall", {
        options: {
          installDir,
          removeUserData,
        },
      });
      setUninstallOutcome(nextOutcome);
      setProgress({ progress: 100, step: "卸载完成" });
      setScreen("complete");
    } catch (reason) {
      setError(normalizeError(reason));
      setScreen("setup");
    }
  }

  async function finish() {
    const shouldLaunchAfterFinish = mode.isUpdate ? mode.launchAfterInstall : launchAfterInstall;
    if (!isUninstall && shouldLaunchAfterFinish && installOutcome) {
      try {
        await invoke("launch_cliply", { installDir: installOutcome.installDir });
      } catch (reason) {
        setError(normalizeError(reason));
        return;
      }
    }
    await getCurrentWindow().close();
  }

  return (
    <div className="installer-root">
      <div className="titlebar" data-tauri-drag-region>
        <div className="titlebar-brand" data-tauri-drag-region>
          <img src="/cliply-logo.png" alt="" />
          <span>{isUninstall ? "Cliply 卸载程序" : "Cliply 安装程序"}</span>
        </div>
        <div className="window-actions">
          <button
            type="button"
            aria-label="最小化"
            onClick={() => void getCurrentWindow().minimize()}
          >
            <Minus size={15} />
          </button>
          <button
            type="button"
            aria-label="关闭"
            disabled={screen === "working" && !error}
            onClick={() => void getCurrentWindow().close()}
          >
            <X size={15} />
          </button>
        </div>
      </div>

      <main className="installer-window">
        {screen === "setup" &&
          (isUninstall ? (
            <UninstallScreen
              installDir={installDir}
              removeUserData={removeUserData}
              error={error}
              onRemoveUserDataChange={setRemoveUserData}
              onUninstall={() => void uninstall()}
              onCancel={() => void getCurrentWindow().close()}
            />
          ) : (
            <SetupScreen
              isUpdate={isUpdate}
              installDir={installDir}
              desktopShortcut={desktopShortcut}
              startOnLogin={startOnLogin}
              error={error}
              onInstallDirChange={setInstallDir}
              onBrowse={chooseInstallDir}
              onDesktopShortcutChange={setDesktopShortcut}
              onStartOnLoginChange={setStartOnLogin}
              onInstall={() => void install()}
              onCancel={() => void getCurrentWindow().close()}
            />
          ))}

        {screen === "working" && (
          <WorkingScreen
            isUpdate={isUpdate}
            isUninstall={isUninstall}
            removeUserData={removeUserData}
            sourceVersion={mode.sourceVersion}
            targetVersion={mode.targetVersion}
            progress={progress}
            error={error}
          />
        )}

        {screen === "complete" && (
          <CompleteScreen
            isUninstall={isUninstall}
            isUpdate={isUpdate}
            launchAfterInstall={launchAfterInstall}
            userDataRemoved={uninstallOutcome?.userDataRemoved ?? false}
            installDir={installOutcome?.installDir ?? installDir}
            targetVersion={mode.targetVersion}
            error={error}
            onLaunchAfterInstallChange={setLaunchAfterInstall}
            onFinish={() => void finish()}
          />
        )}
      </main>
    </div>
  );
}

type SetupScreenProps = {
  isUpdate: boolean;
  installDir: string;
  desktopShortcut: boolean;
  startOnLogin: boolean;
  error: string | null;
  onInstallDirChange: (value: string) => void;
  onBrowse: () => void;
  onDesktopShortcutChange: (value: boolean) => void;
  onStartOnLoginChange: (value: boolean) => void;
  onInstall: () => void;
  onCancel: () => void;
};

function SetupScreen({
  isUpdate,
  installDir,
  desktopShortcut,
  startOnLogin,
  error,
  onInstallDirChange,
  onBrowse,
  onDesktopShortcutChange,
  onStartOnLoginChange,
  onInstall,
  onCancel,
}: SetupScreenProps) {
  const ModeIcon = isUpdate ? RefreshCw : ShieldCheck;

  return (
    <section className="screen setup-screen">
      <div className="hero-row">
        <div className="logo-wrap">
          <img src="/cliply-logo.png" alt="" />
        </div>
        <div className="hero-copy">
          <div className="hero-heading-line">
            <h1>{isUpdate ? "更新 Cliply" : "安装 Cliply"}</h1>
            <span className="mode-badge">
              <ModeIcon size={14} />
              {isUpdate ? "更新" : "安装"}
            </span>
          </div>
          <p>
            {isUpdate
              ? "更新程序文件，历史记录和应用设置保持不变。"
              : "将在此计算机上安装 Cliply。剪贴板历史记录和应用设置默认存储在本机。"}
          </p>
        </div>
      </div>

      <div className="setup-panel">
        <div className="panel-heading-row">
          <div className="panel-title">
            <HardDrive size={18} />
            <div>
              <strong>安装位置</strong>
              <span>{isUpdate ? "当前安装目录" : "请选择 Cliply 的安装目录"}</span>
            </div>
          </div>
          {isUpdate && (
            <span className="detection-badge">
              <Check size={13} />
              已安装
            </span>
          )}
        </div>
        <label className="sr-only" htmlFor="install-dir">
          安装位置
        </label>
        <div className="path-row">
          <div className="path-input-shell">
            <FolderOpen size={16} />
            <input
              id="install-dir"
              value={installDir}
              spellCheck={false}
              onChange={(event) => onInstallDirChange(event.target.value)}
            />
          </div>
          <button type="button" className="ghost-button" onClick={onBrowse}>
            <FolderOpen size={16} />
            更改
          </button>
        </div>

        {error ? (
          <div className="error-banner" role="alert">{error}</div>
        ) : isUpdate ? (
          <div className="update-note">
            <History size={18} />
            <div className="note-copy">
              <strong>更新范围</strong>
              <span>本次更新仅替换程序文件。剪贴板历史记录、主题和快捷键等设置不会更改。</span>
            </div>
          </div>
        ) : (
          <div className="option-grid">
            <CheckOption
              checked={desktopShortcut}
              icon={<Monitor size={17} />}
              label="创建桌面快捷方式"
              description="在 Windows 桌面创建 Cliply 快捷方式"
              onChange={onDesktopShortcutChange}
            />
            <CheckOption
              checked={startOnLogin}
              icon={<Power size={17} />}
              label="登录时自动启动"
              description="登录 Windows 后自动启动 Cliply"
              onChange={onStartOnLoginChange}
            />
          </div>
        )}

        {!error && (
          <div className="install-facts" aria-label="安装摘要">
            <div>
              <span>安装类型</span>
              <strong>{isUpdate ? "更新现有版本" : "首次安装"}</strong>
            </div>
            <div>
              <span>历史记录与设置</span>
              <strong>{isUpdate ? "保持不变" : "存储于本机"}</strong>
            </div>
            <div>
              <span>默认快捷键</span>
              <strong>Ctrl + Shift + V</strong>
            </div>
          </div>
        )}
      </div>

      <div className="actions">
        <div className="trust-note">
          <LockKeyhole size={14} />
          {isUpdate ? "更新不会更改剪贴板历史记录和应用设置" : "剪贴板历史记录默认存储在本机"}
        </div>
        <div className="action-buttons">
          <button type="button" className="secondary-button" onClick={onCancel}>
            取消
          </button>
          <button type="button" className="primary-button" onClick={onInstall}>
            {isUpdate ? "更新 Cliply" : "安装 Cliply"}
            <ArrowRight size={17} />
          </button>
        </div>
      </div>
    </section>
  );
}

type UninstallScreenProps = {
  installDir: string;
  removeUserData: boolean;
  error: string | null;
  onRemoveUserDataChange: (value: boolean) => void;
  onUninstall: () => void;
  onCancel: () => void;
};

function UninstallScreen({
  installDir,
  removeUserData,
  error,
  onRemoveUserDataChange,
  onUninstall,
  onCancel,
}: UninstallScreenProps) {
  return (
    <section className="screen setup-screen">
      <div className="hero-row">
        <div className="logo-wrap danger">
          <Trash2 size={34} />
        </div>
        <div className="hero-copy">
          <div className="hero-heading-line">
            <h1>卸载 Cliply</h1>
            <span className="mode-badge danger">
              <Trash2 size={14} />
              卸载
            </span>
          </div>
          <p>
            卸载将移除 Cliply 的程序文件、快捷方式和开机启动项。剪贴板历史记录和应用设置默认保留。
          </p>
        </div>
      </div>

      <div className="setup-panel">
        <div className="panel-heading-row">
          <div className="panel-title">
            <HardDrive size={18} />
            <div>
              <strong>当前安装位置</strong>
              <span>程序文件将从此目录移除</span>
            </div>
          </div>
          <span className="detection-badge neutral">
            <Check size={13} />
            已安装
          </span>
        </div>
        <div className="readonly-path">
          <FolderOpen size={16} />
          <span>{installDir}</span>
        </div>

        {error ? (
          <div className="error-banner" role="alert">{error}</div>
        ) : (
          <>
            <div className="update-note neutral">
              <History size={18} />
              <div className="note-copy">
                <strong>数据保留</strong>
                <span>如需同时删除剪贴板历史记录和应用设置，请选择下方选项。</span>
              </div>
            </div>

            <div className="option-grid single">
              <CheckOption
                checked={removeUserData}
                icon={<Trash2 size={17} />}
                label="删除剪贴板历史记录和应用设置"
                description="删除后无法恢复"
                tone="danger"
                onChange={onRemoveUserDataChange}
              />
            </div>
          </>
        )}
      </div>

      <div className="actions">
        <div className="trust-note">
          <LockKeyhole size={14} />
          {removeUserData
            ? "剪贴板历史记录和应用设置将一并删除"
            : "剪贴板历史记录和应用设置将保留"}
        </div>
        <div className="action-buttons">
          <button type="button" className="secondary-button" onClick={onCancel}>
            取消
          </button>
          <button type="button" className="danger-button" onClick={onUninstall}>
            卸载 Cliply
            <Trash2 size={16} />
          </button>
        </div>
      </div>
    </section>
  );
}

type WorkingScreenProps = {
  isUpdate: boolean;
  isUninstall: boolean;
  removeUserData: boolean;
  sourceVersion?: string | null;
  targetVersion?: string | null;
  progress: InstallProgress;
  error: string | null;
};

function WorkingScreen({
  isUpdate,
  isUninstall,
  removeUserData,
  sourceVersion,
  targetVersion,
  progress,
  error,
}: WorkingScreenProps) {
  const title = isUninstall
    ? "正在卸载 Cliply"
    : isUpdate
      ? "正在更新 Cliply"
      : "正在安装 Cliply";

  return (
    <section className="screen process-screen">
      <div className="process-shell">
        <div className={isUninstall ? "installing-mark danger" : "installing-mark"}>
          <Loader2 size={29} />
        </div>
        <div className="process-heading">
          <h1>{title}</h1>
          <p>
            {isUninstall
              ? "正在移除程序文件、快捷方式和开机启动项。"
              : isUpdate
                ? "正在替换程序文件。剪贴板历史记录和应用设置不会更改。"
                : "正在复制程序文件并应用安装选项。"}
          </p>
        </div>

        {isUpdate && targetVersion ? (
          <div className="version-row">
            <span>{sourceVersion ? `v${sourceVersion}` : "当前版本"}</span>
            <ChevronRight size={14} />
            <span>v{targetVersion}</span>
          </div>
        ) : null}

        {error ? (
          <div className="installer-error-panel">
            <div className="error-banner compact" role="alert">{error}</div>
            <div className="mini-actions">
              <button
                type="button"
                className="secondary-button"
                onClick={() => void invoke("open_installer_log_directory")}
              >
                打开日志文件夹
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => void invoke("open_release_page")}
              >
                查看发布页面
              </button>
            </div>
          </div>
        ) : (
          <div className="progress-panel">
            <div className="progress-copy" aria-live="polite">
              <span>{progress.step}</span>
              <strong>{progress.progress}%</strong>
            </div>
            <div
              className="progress-track"
              role="progressbar"
              aria-label={title}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.min(Math.max(progress.progress, 0), 100)}
            >
              <div
                className={isUninstall ? "progress-fill danger" : "progress-fill"}
                style={{ width: `${Math.min(Math.max(progress.progress, 0), 100)}%` }}
              />
            </div>
            <div className="progress-caption">
              <LockKeyhole size={13} />
              {isUninstall
                ? removeUserData
                  ? "剪贴板历史记录和应用设置将一并删除"
                  : "剪贴板历史记录和应用设置将保留"
                : isUpdate
                  ? "剪贴板历史记录和应用设置不会更改"
                  : "正在执行本机安装"}
            </div>
          </div>
        )}
      </div>

      <div className="actions process-actions">
        <div className="trust-note">
          <LockKeyhole size={14} />
          {isUninstall ? "正在移除程序文件" : isUpdate ? "正在更新程序文件" : "正在写入程序文件"}
        </div>
        {!error && <div className="process-lock">请勿关闭安装程序</div>}
      </div>
    </section>
  );
}

type CompleteScreenProps = {
  isUninstall: boolean;
  isUpdate: boolean;
  launchAfterInstall: boolean;
  userDataRemoved: boolean;
  installDir: string;
  targetVersion?: string | null;
  error: string | null;
  onLaunchAfterInstallChange: (value: boolean) => void;
  onFinish: () => void;
};

function CompleteScreen({
  isUninstall,
  isUpdate,
  launchAfterInstall,
  userDataRemoved,
  installDir,
  targetVersion,
  error,
  onLaunchAfterInstallChange,
  onFinish,
}: CompleteScreenProps) {
  return (
    <section className="screen process-screen">
      <div className="result-shell">
        <div className="result-heading">
          <div className={isUninstall ? "complete-mark danger" : "complete-mark"}>
            <CircleCheck size={38} />
          </div>
          <div>
            <h1>{isUninstall ? "Cliply 卸载完成" : isUpdate ? "Cliply 更新完成" : "Cliply 安装完成"}</h1>
            <p className="muted-copy">
              {isUninstall
                ? userDataRemoved
                  ? "程序文件、剪贴板历史记录和应用设置已删除。"
                  : "程序文件已移除，剪贴板历史记录和应用设置已保留。"
              : isUpdate
                ? "程序文件已更新，剪贴板历史记录和应用设置已保留。"
                : "Cliply 已安装。使用 Ctrl + Shift + V 可打开剪贴板历史记录。"}
            </p>
          </div>
        </div>

        <div className="result-facts">
          <div>
            <span>操作结果</span>
            <strong>{isUninstall ? "程序已移除" : isUpdate ? "更新完成" : "安装完成"}</strong>
          </div>
          <div>
            <span>{isUninstall ? "历史记录与设置" : "当前版本"}</span>
            <strong>
              {isUninstall
                ? userDataRemoved ? "已删除" : "已保留"
                : targetVersion ? `v${targetVersion}` : "已安装版本"}
            </strong>
          </div>
          <div>
            <span>{isUninstall ? "原安装位置" : "安装位置"}</span>
            <strong title={installDir}>{installDir}</strong>
          </div>
        </div>

        {!isUninstall && !isUpdate && (
          <div className="finish-option">
            <CheckOption
              checked={launchAfterInstall}
              icon={<Power size={17} />}
              label="启动 Cliply"
              description="关闭安装程序后自动启动"
              onChange={onLaunchAfterInstallChange}
            />
          </div>
        )}

        {error && <div className="error-banner compact" role="alert">{error}</div>}
      </div>

      <div className="actions process-actions">
        <div className="trust-note success">
          <CircleCheck size={14} />
          {isUninstall ? "卸载已完成" : isUpdate ? "更新已完成" : "安装已完成"}
        </div>
        <div className="action-buttons">
          <button type="button" className="primary-button" onClick={onFinish}>
            完成
            <ArrowRight size={16} />
          </button>
        </div>
      </div>
    </section>
  );
}

type CheckOptionProps = {
  checked: boolean;
  icon?: React.ReactNode;
  label: string;
  description?: string;
  tone?: "default" | "danger";
  onChange: (checked: boolean) => void;
};

function CheckOption({
  checked,
  icon,
  label,
  description,
  tone = "default",
  onChange,
}: CheckOptionProps) {
  return (
    <label className={`check-option ${tone === "danger" ? "danger" : ""}`}>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="box">{checked && <Check size={13} />}</span>
      {icon && <span className="option-icon">{icon}</span>}
      <span className="option-copy">
        <strong>{label}</strong>
        {description && <span>{description}</span>}
      </span>
    </label>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
