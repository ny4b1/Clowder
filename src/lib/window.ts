import { invoke } from "@tauri-apps/api/core";

export function setWindowFullscreen(fullscreen: boolean) {
  return invoke("set_window_fullscreen", { fullscreen });
}
