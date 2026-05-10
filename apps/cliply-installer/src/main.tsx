import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Check,
  ChevronRight,
  FolderOpen,
  Loader2,
  Minus,
  MonitorUp,
  Sparkles,
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
    step: "准备安装",
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
      .catch((reason) => setError(String(reason)));

    void invoke<InstallDetection>("detect_installation")
      .then((nextDetection) => {
        setDetection(nextDetection);
        if (!modeInstallDirRef.current) {
          setInstallDir(nextDetection.installDir);
        }
      })
      .catch((reason) => setError(String(reason)));

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
      setError(String(reason));
    }
  }

  async function install() {
    setError(null);
    setProgress({
      progress: 0,
      step: isUpdate ? "准备更新 Cliply" : "准备安装 Cliply",
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
          launchAfterInstall: mode.launchAfterInstall,
          parentPid: mode.parentPid ?? null,
        },
      });
      setInstallOutcome(nextOutcome);
      setProgress({ progress: 100, step: "安装完成" });
      setScreen("complete");
    } catch (reason) {
      setError(String(reason));
      setScreen(mode.isUpdate ? "working" : "setup");
    }
  }

  async function uninstall() {
    setError(null);
    setProgress({ progress: 0, step: "准备卸载 Cliply" });
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
      setError(String(reason));
      setScreen("setup");
    }
  }

  async function finish() {
    if (!isUninstall && !(mode.isUpdate && mode.launchAfterInstall) && launchAfterInstall && installOutcome) {
      try {
        await invoke("launch_cliply", { installDir: installOutcome.installDir });
      } catch (reason) {
        setError(String(reason));
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
          <span>{isUninstall ? "Cliply Uninstaller" : "Cliply Installer"}</span>
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
            sourceVersion={mode.sourceVersion}
            targetVersion={mode.targetVersion}
            progress={progress}
            error={error}
            onCancel={() => void getCurrentWindow().close()}
          />
        )}

        {screen === "complete" && (
          <CompleteScreen
            isUninstall={isUninstall}
            isUpdate={isUpdate}
            launchHandledByInstaller={mode.isUpdate && mode.launchAfterInstall}
            launchAfterInstall={launchAfterInstall}
            userDataRemoved={uninstallOutcome?.userDataRemoved ?? false}
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
  return (
    <section className="screen setup-screen">
      <div className="hero-row">
        <div className="logo-wrap">
          <img src="/cliply-logo.png" alt="" />
        </div>
        <div>
          <div className="eyebrow">Local-first clipboard manager</div>
          <h1>{isUpdate ? "更新 Cliply" : "安装 Cliply"}</h1>
          <p>
            快速找回复制过的文本、链接、图片和代码。所有数据默认保存在本地。
          </p>
        </div>
      </div>

      <div className="setup-panel">
        <label className="field-label" htmlFor="install-dir">
          安装位置
        </label>
        <div className="path-row">
          <input
            id="install-dir"
            value={installDir}
            spellCheck={false}
            onChange={(event) => onInstallDirChange(event.target.value)}
          />
          <button type="button" className="ghost-button" onClick={onBrowse}>
            <FolderOpen size={16} />
            更改
          </button>
        </div>

        {isUpdate && (
          <div className="update-note">
            <MonitorUp size={16} />
            检测到已安装的 Cliply。本次将覆盖更新程序文件，并保留本地历史记录与设置。
          </div>
        )}

        <div className="option-grid">
          <CheckOption
            checked={desktopShortcut}
            label="创建桌面快捷方式"
            onChange={onDesktopShortcutChange}
          />
          <CheckOption
            checked={startOnLogin}
            label="开机自动启动"
            onChange={onStartOnLoginChange}
          />
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className="actions">
        <button type="button" className="secondary-button" onClick={onCancel}>
          取消
        </button>
        <button type="button" className="primary-button" onClick={onInstall}>
          {isUpdate ? "更新 Cliply" : "安装 Cliply"}
          <ChevronRight size={17} />
        </button>
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
        <div>
          <div className="eyebrow">Cliply Uninstaller</div>
          <h1>卸载 Cliply</h1>
          <p>
            将移除 Cliply 程序文件、开始菜单快捷方式和开机启动项。本地历史记录与设置默认保留。
          </p>
        </div>
      </div>

      <div className="setup-panel">
        <div className="field-label">安装位置</div>
        <div className="readonly-path">{installDir}</div>

        <div className="update-note neutral">
          保留本地数据后，重新安装 Cliply 会继续使用原来的剪贴板历史和设置。
        </div>

        <div className="option-grid single">
          <CheckOption
            checked={removeUserData}
            label="同时删除本地历史记录与设置"
            onChange={onRemoveUserDataChange}
          />
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className="actions">
        <button type="button" className="secondary-button" onClick={onCancel}>
          取消
        </button>
        <button type="button" className="danger-button" onClick={onUninstall}>
          卸载 Cliply
          <Trash2 size={16} />
        </button>
      </div>
    </section>
  );
}

type WorkingScreenProps = {
  isUpdate: boolean;
  isUninstall: boolean;
  sourceVersion?: string | null;
  targetVersion?: string | null;
  progress: InstallProgress;
  error: string | null;
  onCancel: () => void;
};

function WorkingScreen({
  isUpdate,
  isUninstall,
  sourceVersion,
  targetVersion,
  progress,
  error,
  onCancel,
}: WorkingScreenProps) {
  const title = isUninstall
    ? "正在卸载 Cliply"
    : isUpdate
      ? "正在更新 Cliply"
      : "正在安装 Cliply";

  return (
    <section className="screen centered-screen">
      <div className={isUninstall ? "installing-mark danger" : "installing-mark"}>
        <Loader2 size={30} />
      </div>
      <h1>{title}</h1>
      {isUpdate && targetVersion ? (
        <div className="version-row">
          <span>{sourceVersion ? `v${sourceVersion}` : "当前版本"}</span>
          <ChevronRight size={14} />
          <span>v{targetVersion}</span>
        </div>
      ) : null}
      <p className="muted-copy">{progress.step}</p>

      <div className="progress-track" aria-label="进度">
        <div
          className={isUninstall ? "progress-fill danger" : "progress-fill"}
          style={{ width: `${Math.min(Math.max(progress.progress, 0), 100)}%` }}
        />
      </div>
      <div className="progress-value">{progress.progress}%</div>

      {error ? (
        <div className="installer-error-panel">
          <div className="error-banner compact">{error}</div>
          <div className="mini-actions">
            <button
              type="button"
              className="secondary-button"
              onClick={() => void invoke("open_installer_log_directory")}
            >
              打开日志目录
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => void invoke("open_release_page")}
            >
              打开 Release 页面
            </button>
          </div>
        </div>
      ) : null}

      <div className="actions single-action">
        <button type="button" className="secondary-button" onClick={onCancel}>
          取消
        </button>
      </div>
    </section>
  );
}

type CompleteScreenProps = {
  isUninstall: boolean;
  isUpdate: boolean;
  launchHandledByInstaller: boolean;
  launchAfterInstall: boolean;
  userDataRemoved: boolean;
  error: string | null;
  onLaunchAfterInstallChange: (value: boolean) => void;
  onFinish: () => void;
};

function CompleteScreen({
  isUninstall,
  isUpdate,
  launchHandledByInstaller,
  launchAfterInstall,
  userDataRemoved,
  error,
  onLaunchAfterInstallChange,
  onFinish,
}: CompleteScreenProps) {
  return (
    <section className="screen centered-screen">
      <div className={isUninstall ? "complete-mark danger" : "complete-mark"}>
        <Check size={34} />
      </div>
      <h1>{isUninstall ? "Cliply 已卸载" : isUpdate ? "Cliply 已更新" : "Cliply 已准备就绪"}</h1>
      <p className="muted-copy">
        {isUninstall
          ? userDataRemoved
            ? "本地历史记录与设置已删除。"
            : "本地历史记录与设置已保留，重新安装后仍可继续使用。"
          : isUpdate
            ? "用户数据已保留，Cliply 会重新启动。"
          : "使用 Ctrl + Shift + V 打开剪贴板历史。"}
      </p>

      {!isUninstall && !launchHandledByInstaller && (
        <div className="finish-option">
          <CheckOption
            checked={launchAfterInstall}
            label="立即启动 Cliply"
            onChange={onLaunchAfterInstallChange}
          />
        </div>
      )}

      {error && <div className="error-banner compact">{error}</div>}

      <div className="actions single-action">
        <button type="button" className="primary-button" onClick={onFinish}>
          完成
          <Sparkles size={16} />
        </button>
      </div>
    </section>
  );
}

type CheckOptionProps = {
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
};

function CheckOption({ checked, label, onChange }: CheckOptionProps) {
  return (
    <label className="check-option">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="box">{checked && <Check size={13} />}</span>
      <span>{label}</span>
    </label>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
