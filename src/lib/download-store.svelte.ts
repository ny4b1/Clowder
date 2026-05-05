import { downloadFile, originalUrl } from "./e621";
import { settingsStore } from "./settings-store.svelte";
import { applyFilenameTemplate } from "./template";
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
    this.pending = { ...this.pending };
    this.status = "downloading";

    try {
      const template =
        settingsStore.current.downloads.filename_template.trim() || "{artist}_{id}.{ext}";
      const filename = applyFilenameTemplate(template, post);
      const path = await downloadFile(url, filename);
      this.status = `saved ${path}`;
    } catch (error) {
      this.status = `download failed: ${String(error)}`;
    } finally {
      delete this.pending[post.id];
      this.pending = { ...this.pending };
    }
  }
}

export const downloadStore = new DownloadStore();
