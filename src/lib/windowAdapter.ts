import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "@tauri-apps/api/core";

export async function hideMainWindow() {
  if (!isTauri()) {
    console.info("[cliply:mock-window] hide main window");
    return;
  }

  await getCurrentWindow().hide();
}

export async function toggleAlwaysOnTop(nextValue: boolean) {
  if (!isTauri()) {
    console.info("[cliply:mock-window] always on top", nextValue);
    return;
  }

  await getCurrentWindow().setAlwaysOnTop(nextValue);
}
