import { check, type Update } from "@tauri-apps/plugin-updater";
import { errMsg } from "./errors";
import { toastStore } from "./toast-store.svelte";

type Status = "idle" | "checking" | "available" | "downloading" | "ready" | "error";

class UpdateStore {
  status = $state<Status>("idle");
  available = $state<Update | null>(null);
  error = $state<string | null>(null);
  progress = $state<{ downloaded: number; total: number | null }>({ downloaded: 0, total: null });

  async check(silent = false): Promise<void> {
    if (this.status === "checking" || this.status === "downloading") return;
    this.status = "checking";
    this.error = null;
    try {
      const update = await check();
      if (update) {
        this.available = update;
        this.status = "available";
        if (silent) {
          toastStore.show(`Clowder ${update.version} is available — open Settings → About to install.`);
        }
      } else {
        this.available = null;
        this.status = "idle";
      }
    } catch (error) {
      this.status = "error";
      const message = errMsg(error);
      this.error = message;
      if (!silent) toastStore.error(`Update check failed: ${message}`);
    }
  }

  async install(): Promise<void> {
    const update = this.available;
    if (!update || this.status === "downloading") return;
    this.status = "downloading";
    this.error = null;
    this.progress = { downloaded: 0, total: null };
    try {
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            this.progress = { downloaded: 0, total: event.data.contentLength ?? null };
            break;
          case "Progress":
            this.progress = {
              downloaded: this.progress.downloaded + event.data.chunkLength,
              total: this.progress.total,
            };
            break;
          case "Finished":
            this.status = "ready";
            toastStore.success(`Clowder ${update.version} installed — restart to apply.`);
            break;
        }
      });
    } catch (error) {
      this.status = "error";
      const message = errMsg(error);
      this.error = message;
      toastStore.error(`Update failed: ${message}`);
    }
  }
}

export const updateStore = new UpdateStore();
