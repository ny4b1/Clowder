<script lang="ts">
  import VideoControls from "./VideoControls.svelte";

  type VideoUi = {
    currentTime: number;
    duration: number;
    paused: boolean;
    muted: boolean;
    volume: number;
  };

  type Props = {
    src: string;
    autoplay: boolean;
    appVideoFullscreen: boolean;
    videoUi: VideoUi;
    showControls: boolean;
    showMenu: boolean;
    videoElement?: HTMLVideoElement;
    frameElement?: HTMLDivElement;

    onTimeUpdate: (target: HTMLVideoElement) => void;
    onPause: (target: HTMLVideoElement) => void;
    onPlay: (target: HTMLVideoElement) => void;
    onVolumeChange: (target: HTMLVideoElement) => void;
    onLoadedMetadata: (target: HTMLVideoElement) => void;

    onReveal: () => void;
    onHide: () => void;

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
    src,
    autoplay,
    appVideoFullscreen,
    videoUi,
    showControls,
    showMenu,
    videoElement = $bindable(),
    frameElement = $bindable(),
    onTimeUpdate,
    onPause,
    onPlay,
    onVolumeChange,
    onLoadedMetadata,
    onReveal,
    onHide,
    onTogglePlayback,
    onSeek,
    onToggleMute,
    onSetVolume,
    onToggleFullscreen,
    onToggleMenu,
    onSetRate,
    onCopyUrl,
  }: Props = $props();
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  bind:this={frameElement}
  class={appVideoFullscreen ? "fixed inset-0 z-[70] bg-black" : "absolute inset-4"}
  onclick={(event) => event.stopPropagation()}
  onmousemove={onReveal}
  onmouseenter={onReveal}
  onmouseleave={onHide}
>
  <video
    bind:this={videoElement}
    class="h-full w-full object-contain"
    {src}
    {autoplay}
    loop
    ontimeupdate={(event) => onTimeUpdate(event.currentTarget)}
    onpause={(event) => onPause(event.currentTarget)}
    onplay={(event) => onPlay(event.currentTarget)}
    onvolumechange={(event) => onVolumeChange(event.currentTarget)}
    onloadedmetadata={(event) => onLoadedMetadata(event.currentTarget)}
  ></video>
  <VideoControls
    target={videoElement}
    frame={frameElement}
    {videoUi}
    {showControls}
    {showMenu}
    {onReveal}
    {onTogglePlayback}
    {onSeek}
    {onToggleMute}
    {onSetVolume}
    {onToggleFullscreen}
    {onToggleMenu}
    {onSetRate}
    {onCopyUrl}
  />
</div>
