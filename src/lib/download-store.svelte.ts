import { downloadFile, originalUrl } from "./e621";
import { errMsg } from "./errors";
import { settingsStore } from "./settings-store.svelte";
import { DEFAULT_FILENAME_TEMPLATE, applyFilenameTemplate } from "./template";
import { toastStore } from "./toast-store.svelte";
import type { Post } from "./types";

class DownloadStore {
  pending = $state<Record<number, boolean>>({});
  status = $state("");

  isPending(id: number): boolean {
    return !!this.pending[id];
  }

  reset() {
    this.status = "";
  }

  async download(post: Post): Promise<void> {
    const url = originalUrl(post);
    if (!url || this.pending[post.id]) return;

    this.pending[post.id] = true;
    this.status = "downloading";

    try {
      const template =
        settingsStore.current.downloads.filename_template.trim() || DEFAULT_FILENAME_TEMPLATE;
      const filename = applyFilenameTemplate(template, post);
      const path = await downloadFile(url, filename);
      this.status = `saved ${path}`;
      toastStore.success(`saved ${path}`);
    } catch (error) {
      const message = `download failed: ${errMsg(error)}`;
      this.status = message;
      toastStore.error(message);
    } finally {
      delete this.pending[post.id];
    }
  }
}

export const downloadStore = new DownloadStore();
