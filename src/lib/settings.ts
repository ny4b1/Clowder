import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "./types";

export function getSettings() {
  return invoke<Settings>("get_settings");
}

export function updateSettings(settings: Settings) {
  return invoke<Settings>("update_settings", { newSettings: settings });
}
