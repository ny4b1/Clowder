import { invoke } from "@tauri-apps/api/core";
import type { MullvadCountry, VpnStatus } from "./types";

export function importVpnConfig(path: string) {
  return invoke<VpnStatus>("import_vpn_config", { path });
}

export function mullvadSignIn(account: string) {
  return invoke<VpnStatus>("mullvad_sign_in", { account });
}

export function mullvadLocations() {
  return invoke<MullvadCountry[]>("mullvad_locations");
}

export function mullvadSelectRelay(cityCode: string) {
  return invoke<VpnStatus>("mullvad_select_relay", { cityCode });
}

export function mullvadSignOut() {
  return invoke<VpnStatus>("mullvad_sign_out");
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
