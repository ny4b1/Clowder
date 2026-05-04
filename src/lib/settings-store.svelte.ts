import { getSettings, updateSettings } from "./settings";
import type { Settings } from "./types";

const defaults: Settings = {
  doh_provider: "cloudflare",
  fail_closed_ech: false,
  downloads: {
    directory: null,
    filename_template: "{artist}_{id}.{ext}",
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
      this.error = String(error);
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
