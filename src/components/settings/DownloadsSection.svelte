<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { errMsg } from "../../lib/errors";
  import {
    DEFAULT_FILENAME_TEMPLATE,
    FILENAME_TOKENS,
    applyFilenameTemplate,
  } from "../../lib/template";
  import type { DownloadSettings } from "../../lib/types";

  type Props = {
    downloads: DownloadSettings;
    saving: boolean;
    onError: (message: string) => void;
  };

  let { downloads = $bindable(), saving, onError }: Props = $props();

  let templatePreview = $derived(
    applyFilenameTemplate(downloads.filename_template.trim() || DEFAULT_FILENAME_TEMPLATE, {
      id: 12345,
      file: { ext: "png" },
      tags: { artist: ["sample_artist"] },
      score: { up: 100, down: 5, total: 95 },
      fav_count: 42,
    }),
  );

  async function pickDownloadDir() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select download folder",
        defaultPath: downloads.directory ?? undefined,
      });
      if (typeof selected === "string" && selected) {
        downloads.directory = selected;
      }
    } catch (error) {
      onError(errMsg(error));
    }
  }
</script>

<div class="space-y-5 px-5 py-4">
  <div class="space-y-2">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      download folder
    </div>
    <div
      class="flex items-stretch overflow-hidden rounded-[3px] border border-room-line bg-room-panel"
    >
      <span
        class="min-w-0 flex-1 truncate px-3 py-2 font-mono text-[11.5px] text-room-text"
        title={downloads.directory ?? "~/Downloads/Clowder (default)"}
      >
        {downloads.directory ?? "~/Downloads/Clowder (default)"}
      </span>
      <button
        type="button"
        onclick={pickDownloadDir}
        disabled={saving}
        class="border-l border-room-line px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text disabled:opacity-50"
      >
        choose
      </button>
      {#if downloads.directory}
        <button
          type="button"
          onclick={() => (downloads.directory = null)}
          disabled={saving}
          class="border-l border-room-line px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text disabled:opacity-50"
        >
          reset
        </button>
      {/if}
    </div>
  </div>

  <div class="space-y-2 border-t border-room-line pt-4">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      filename template
    </div>
    <input
      value={downloads.filename_template}
      oninput={(event) => (downloads.filename_template = event.currentTarget.value)}
      disabled={saving}
      spellcheck="false"
      class="block h-8 w-full rounded-[3px] border border-room-line bg-room-panel px-2.5 font-mono text-[12px] text-room-text outline-none transition-colors duration-150 focus:border-room-accent disabled:opacity-50"
    />
    <div class="flex flex-wrap gap-1">
      {#each FILENAME_TOKENS as item (item.token)}
        <button
          type="button"
          onclick={() => (downloads.filename_template = downloads.filename_template + item.token)}
          disabled={saving}
          class="inline-flex h-6 items-center rounded-[2px] border border-room-line bg-room-panel px-2 font-mono text-[10.5px] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent disabled:opacity-50"
          title={item.description}
        >
          {item.token}
        </button>
      {/each}
    </div>
    <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
      preview: <span class="text-room-text-mid">{templatePreview}</span>
    </p>
    <p class="font-mono text-[10px] leading-relaxed text-room-text-low">
      Existing files at the same path get a numeric suffix (-1, -2, ...) automatically.
    </p>
  </div>
</div>
