import { invoke } from "@tauri-apps/api/core";
import type { DohProvider, Settings } from "./types";

export function getSettings() {
  return invoke<Settings>("get_settings");
}

export function updateSettings(settings: Settings) {
  return invoke<Settings>("update_settings", { newSettings: settings });
}

export const dohProviderOptions: { value: DohProvider; label: string; description: string }[] = [
  {
    value: "cloudflare",
    label: "Cloudflare",
    description: "1.1.1.1 — fast, widely available",
  },
  {
    value: "google",
    label: "Google",
    description: "8.8.8.8 — broad reach, large logging surface",
  },
  {
    value: "quad9",
    label: "Quad9",
    description: "9.9.9.9 — Swiss non-profit, blocks known malicious domains",
  },
  {
    value: "ad_guard",
    label: "AdGuard",
    description: "94.140.14.14 — blocks ads and trackers at DNS level",
  },
];
