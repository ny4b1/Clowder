<script lang="ts">
  type VideoUi = {
    currentTime: number;
    duration: number;
    paused: boolean;
    muted: boolean;
    volume: number;
  };

  type Props = {
    target: HTMLVideoElement | undefined;
    frame: HTMLDivElement | undefined;
    videoUi: VideoUi;
    showControls: boolean;
    showMenu: boolean;
    onReveal: () => void;
    onTogglePlayback: (target: HTMLVideoElement | undefined) => void | Promise<void>;
    onSeek: (target: HTMLVideoElement | undefined, value: string) => void;
    onToggleMute: (target: HTMLVideoElement | undefined) => void;
    onSetVolume: (target: HTMLVideoElement | undefined, value: string) => void;
    onToggleFullscreen: (
      target: HTMLVideoElement | undefined,
      frame: HTMLDivElement | undefined,
    ) => void | Promise<void>;
    onToggleMenu: () => void;
    onSetRate: (target: HTMLVideoElement | undefined, rate: number) => void;
    onCopyUrl: () => void;
  };

  let {
    target,
    frame,
    videoUi,
    showControls,
    showMenu,
    onReveal,
    onTogglePlayback,
    onSeek,
    onToggleMute,
    onSetVolume,
    onToggleFullscreen,
    onToggleMenu,
    onSetRate,
    onCopyUrl,
  }: Props = $props();

  function formatVideoTime(value: number) {
    if (!Number.isFinite(value) || value <= 0) return "0:00";
    const rounded = Math.round(value);
    const minutes = Math.floor(rounded / 60);
    const seconds = rounded % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  function videoProgressValue() {
    const duration = videoUi.duration;
    if (!Number.isFinite(duration) || duration <= 0) return 0;
    if (duration - videoUi.currentTime < 0.25) return duration;
    return Math.min(duration, Math.max(0, videoUi.currentTime));
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute inset-x-0 bottom-0 flex h-11 items-center gap-2 bg-room-floor/90 px-3 text-room-text backdrop-blur transition-opacity duration-200 {showControls || showMenu
    ? 'opacity-100'
    : 'pointer-events-none opacity-0'}"
  onmousemove={onReveal}
>
  <button
    type="button"
    onclick={() => void onTogglePlayback(target)}
    class="flex size-8 items-center justify-center rounded-[3px] text-room-text-mid transition-colors hover:text-room-text"
    aria-label={videoUi.paused ? "Play video" : "Pause video"}
  >
    {#if videoUi.paused}
      <svg class="size-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M8 5v14l11-7z" />
      </svg>
    {:else}
      <svg class="size-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M6 5h4v14H6zM14 5h4v14h-4z" />
      </svg>
    {/if}
  </button>
  <span class="w-20 font-mono text-[10px] tabular-nums text-room-text-low">
    {formatVideoTime(videoUi.currentTime)} / {formatVideoTime(videoUi.duration)}
  </span>
  <input
    type="range"
    min="0"
    max={videoUi.duration || 0}
    step="0.05"
    value={videoProgressValue()}
    oninput={(event) => onSeek(target, event.currentTarget.value)}
    class="min-w-0 flex-1 accent-room-accent"
    aria-label="Video position"
  />
  <button
    type="button"
    onclick={() => onToggleMute(target)}
    class="flex size-8 items-center justify-center rounded-[3px] text-room-text-mid transition-colors hover:text-room-text"
    aria-label={videoUi.muted ? "Unmute video" : "Mute video"}
  >
    {#if videoUi.muted || videoUi.volume === 0}
      <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M11 5 6 9H2v6h4l5 4z" />
        <path d="m23 9-6 6" />
        <path d="m17 9 6 6" />
      </svg>
    {:else}
      <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M11 5 6 9H2v6h4l5 4z" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
      </svg>
    {/if}
  </button>
  <input
    type="range"
    min="0"
    max="1"
    step="0.05"
    value={videoUi.muted ? 0 : videoUi.volume}
    oninput={(event) => onSetVolume(target, event.currentTarget.value)}
    class="w-20 accent-room-accent"
    aria-label="Video volume"
  />
  <button
    type="button"
    onclick={() => void onToggleFullscreen(target, frame)}
    class="flex size-8 items-center justify-center rounded-[3px] text-room-text-mid transition-colors hover:text-room-text"
    aria-label="Toggle video fullscreen"
  >
    <svg
      class="size-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="m15 3 6 0 0 6" />
      <path d="M21 3 14 10" />
      <path d="m9 21-6 0 0-6" />
      <path d="M3 21 10 14" />
    </svg>
  </button>
  <button
    type="button"
    onclick={onToggleMenu}
    class="flex size-8 items-center justify-center rounded-[3px] text-room-text-mid transition-colors hover:text-room-text"
    aria-label="More video actions"
  >
    <svg class="size-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <circle cx="5" cy="12" r="1.8" />
      <circle cx="12" cy="12" r="1.8" />
      <circle cx="19" cy="12" r="1.8" />
    </svg>
  </button>
  {#if showMenu}
    <div
      class="absolute bottom-11 right-2 w-44 rounded-[3px] border border-room-line bg-room-panel-hi py-1 shadow-[0_8px_24px_rgba(0,0,0,0.45)]"
    >
      <button
        type="button"
        onclick={() => onSetRate(target, 0.5)}
        class="block w-full px-3 py-1.5 text-left font-mono text-[10.5px] uppercase tracking-[0.14em] text-room-text-mid hover:bg-room-bg hover:text-room-text"
      >
        speed 0.5x
      </button>
      <button
        type="button"
        onclick={() => onSetRate(target, 1)}
        class="block w-full px-3 py-1.5 text-left font-mono text-[10.5px] uppercase tracking-[0.14em] text-room-text-mid hover:bg-room-bg hover:text-room-text"
      >
        speed 1x
      </button>
      <button
        type="button"
        onclick={() => onSetRate(target, 2)}
        class="block w-full px-3 py-1.5 text-left font-mono text-[10.5px] uppercase tracking-[0.14em] text-room-text-mid hover:bg-room-bg hover:text-room-text"
      >
        speed 2x
      </button>
      <button
        type="button"
        onclick={onCopyUrl}
        class="block w-full border-t border-room-line px-3 py-1.5 text-left font-mono text-[10.5px] uppercase tracking-[0.14em] text-room-text-mid hover:bg-room-bg hover:text-room-text"
      >
        copy URL
      </button>
    </div>
  {/if}
</div>
