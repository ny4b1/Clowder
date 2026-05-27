<script lang="ts">
  import type { PlaybackSettings } from "../../lib/types";

  type Props = {
    playback: PlaybackSettings;
    saving: boolean;
  };

  let { playback = $bindable(), saving }: Props = $props();

  const chunkPresets = [
    { value: 2, label: "2 MB" },
    { value: 4, label: "4 MB" },
    { value: 8, label: "8 MB" },
    { value: 16, label: "16 MB" },
  ];
</script>

<div class="space-y-5 px-5 py-4">
  <div class="space-y-2">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      video behavior
    </div>
    <label
      class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {playback.autoplay
        ? 'border-room-accent'
        : 'border-room-line hover:border-room-line-strong'}"
    >
      <input
        type="checkbox"
        checked={playback.autoplay}
        disabled={saving}
        onchange={(event) => (playback.autoplay = event.currentTarget.checked)}
        class="mt-1 accent-room-accent"
      />
      <span class="min-w-0 flex-1">
        <span class="block text-[12.5px] text-room-text">Autoplay videos</span>
        <span
          class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
        >
          Start playback automatically when opening a video post.
        </span>
      </span>
    </label>
    <label
      class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {playback.remember_volume
        ? 'border-room-accent'
        : 'border-room-line hover:border-room-line-strong'}"
    >
      <input
        type="checkbox"
        checked={playback.remember_volume}
        disabled={saving}
        onchange={(event) => (playback.remember_volume = event.currentTarget.checked)}
        class="mt-1 accent-room-accent"
      />
      <span class="min-w-0 flex-1">
        <span class="block text-[12.5px] text-room-text">Remember volume</span>
        <span
          class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
        >
          Restore the last volume / mute setting when opening a new video.
        </span>
      </span>
    </label>
  </div>

  <div class="space-y-2 border-t border-room-line pt-4">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      video chunk size
    </div>
    <p class="text-[11.5px] leading-relaxed text-room-text-mid">
      How much of a video file Clowder fetches per range request. Larger values reduce buffering
      pauses but use more memory and bandwidth per seek.
    </p>
    <div class="flex flex-wrap gap-1.5">
      {#each chunkPresets as preset (preset.value)}
        {@const isActive = playback.video_chunk_mb === preset.value}
        <button
          type="button"
          onclick={() => (playback.video_chunk_mb = preset.value)}
          disabled={saving}
          class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
            ? 'border-room-accent text-room-accent'
            : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
        >
          {preset.label}
        </button>
      {/each}
    </div>
  </div>
</div>
