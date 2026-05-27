import {
  clearVpnConfig,
  disableVpn,
  enableVpn,
  getVpnStatus,
  importVpnConfig,
} from "./vpn";
import { errMsg } from "./errors";
import type { VpnStatus } from "./types";

const EMPTY_STATUS: VpnStatus = {
  configured: false,
  enabled: false,
  endpoint: null,
  proxy_url: null,
};

class VpnStore {
  status = $state<VpnStatus>(EMPTY_STATUS);
  loaded = $state(false);
  busy = $state(false);
  error = $state("");

  async refresh() {
    try {
      this.status = await getVpnStatus();
      this.loaded = true;
      this.error = "";
    } catch (error) {
      this.error = errMsg(error);
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
