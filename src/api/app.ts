import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

export function getAppVersion(): Promise<string> {
  return getVersion();
}

export function markFrontendReady(): Promise<void> {
  return invoke("frontend_ready");
}

export function markPageLoaded(page: string): Promise<void> {
  return invoke("frontend_page_loaded", { page });
}
