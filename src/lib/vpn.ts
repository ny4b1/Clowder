import { invoke } from "@tauri-apps/api/core";
import type { VpnStatus } from "./types";

export function importVpnConfig(path: string) {
  return invoke<VpnStatus>("import_vpn_config", { path });
}

export function enableVpn() {
  return invoke<VpnStatus>("enable_vpn");
}

export function disableVpn() {
  return invoke<VpnStatus>("disable_vpn");
}

export function clearVpnConfig() {
  return invoke<VpnStatus>("clear_vpn_config");
}

export function getVpnStatus() {
  return invoke<VpnStatus>("get_vpn_status");
}
