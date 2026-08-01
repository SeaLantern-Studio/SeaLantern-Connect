import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

const mainWindow = getCurrentWindow();

export function closeWindow(): Promise<void> {
  return mainWindow.close();
}

export function minimizeWindow(): Promise<void> {
  return mainWindow.minimize();
}

export function toggleMaximize(): Promise<void> {
  return mainWindow.toggleMaximize();
}

export function isWindowMaximized(): Promise<boolean> {
  return mainWindow.isMaximized();
}

export function onWindowResized(callback: () => void): Promise<() => void> {
  return mainWindow.onResized(callback);
}

export function onCloseActionRequested(callback: () => void): Promise<() => void> {
  return listen("close-action-requested", callback);
}
