import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke, isTauri } from "@tauri-apps/api/core";

export async function hideMainWindow() {
  if (!isTauri()) {
    console.info("[cliply:mock-window] hide main window");
    return;
  }

  await invoke("hide_main_window");
}

export async function toggleAlwaysOnTop(nextValue: boolean) {
  if (!isTauri()) {
    console.info("[cliply:mock-window] always on top", nextValue);
    return;
  }

  await invoke("toggle_main_window_pin", { pinned: nextValue });
}

export async function toggleMainWindowMaximize() {
  if (!isTauri()) {
    console.info("[cliply:mock-window] toggle maximize");
    return;
  }

  await getCurrentWindow().toggleMaximize();
}

export async function showMainWindow() {
  if (!isTauri()) {
    console.info("[cliply:mock-window] show main window");
    return;
  }

  await invoke("show_main_window");
}

export async function hideCurrentWindowFallback() {
  if (!isTauri()) {
    return;
  }

  await getCurrentWindow().hide();
}
