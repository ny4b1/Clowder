import {
  clearVpnConfig,
  disableVpn,
  enableVpn,
  getVpnStatus,
  importVpnConfig,
  mullvadLocations,
  mullvadSelectRelay,
  mullvadSignIn,
  mullvadSignOut,
} from "./vpn";
import { errMsg } from "./errors";
import type { MullvadCountry, VpnStatus } from "./types";

const EMPTY_STATUS: VpnStatus = {
  configured: false,
  enabled: false,
  endpoint: null,
  proxy_url: null,
  provider: null,
  account: null,
  device: null,
  country: null,
  country_code: null,
  city: null,
  city_code: null,
};

class VpnStore {
  status = $state<VpnStatus>(EMPTY_STATUS);
  loaded = $state(false);
  busy = $state(false);
  error = $state("");
  locations = $state<MullvadCountry[]>([]);
  locationsLoaded = $state(false);

  async refresh() {
    try {
      this.status = await getVpnStatus();
      this.loaded = true;
      this.error = "";
    } catch (error) {
      this.error = errMsg(error);
    }
  }

  async signInMullvad(account: string) {
    this.busy = true;
    this.error = "";
    try {
      this.status = await mullvadSignIn(account);
      this.loaded = true;
      await this.loadLocations(true);
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async loadLocations(force = false) {
    if (this.locationsLoaded && !force) return;
    try {
      this.locations = await mullvadLocations();
      this.locationsLoaded = true;
    } catch (error) {
      this.error = errMsg(error);
    }
  }

  async selectRelay(cityCode: string) {
    this.busy = true;
    this.error = "";
    try {
      this.status = await mullvadSelectRelay(cityCode);
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async signOutMullvad() {
    this.busy = true;
    this.error = "";
    try {
      this.status = await mullvadSignOut();
      this.locations = [];
      this.locationsLoaded = false;
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async import(path: string) {
    this.busy = true;
    this.error = "";
    try {
      this.status = await importVpnConfig(path);
      this.loaded = true;
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async enable() {
    this.busy = true;
    this.error = "";
    try {
      this.status = await enableVpn();
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async disable() {
    this.busy = true;
    this.error = "";
    try {
      this.status = await disableVpn();
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async clear() {
    this.busy = true;
    this.error = "";
    try {
      this.status = await clearVpnConfig();
      return true;
    } catch (error) {
      this.error = errMsg(error);
      return false;
    } finally {
      this.busy = false;
    }
  }
}

export const vpnStore = new VpnStore();
