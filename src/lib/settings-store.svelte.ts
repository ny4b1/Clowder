import { errMsg } from "./errors";
import { getSettings, updateSettings } from "./settings";
import { DEFAULT_FILENAME_TEMPLATE } from "./template";
import type { Settings } from "./types";

const defaults: Settings = {
  downloads: {
    directory: null,
    filename_template: DEFAULT_FILENAME_TEMPLATE,
  },
  playback: {
    autoplay: true,
    remember_volume: true,
    video_chunk_mb: 2,
  },
  appearance: {
    theme: "system",
    motion: "system",
    grid_min_tile_px: 176,
  },
};

class SettingsStore {
  current = $state<Settings>(defaults);
  loaded = $state(false);
  error = $state<string | null>(null);

  async load() {
    try {
      this.current = await getSettings();
      this.error = null;
    } catch (error) {
      this.error = errMsg(error);
    } finally {
      this.loaded = true;
    }
  }

  async save(next: Settings) {
    const result = await updateSettings(next);
    this.current = result;
    return result;
  }
}

export const settingsStore = new SettingsStore();
