<script lang="ts">
  import { onMount } from "svelte";
  import CommentsPanel from "./CommentsPanel.svelte";
  import OriginalPostSidebar from "./OriginalPostSidebar.svelte";
  import VideoControls from "./VideoControls.svelte";
  import type { CommentState, OriginalViewer as OriginalViewerState, Post } from "../lib/types";
  import { isVideoPost } from "../lib/e621";
  import { playbackMemory } from "../lib/playback.svelte";
  import { dimsLabel, postLabel } from "../lib/search";
  import { settingsStore } from "../lib/settings-store.svelte";
  import { setWindowFullscreen } from "../lib/window";

  type Props = {
    viewer: OriginalViewerState;
    imageOnly: boolean;
    username: string | null;
    favoritePending: boolean;
    downloadPending: boolean;
    downloadStatus: string;
    comments: CommentState;
    onClose: () => void;
    onToggleImageOnly: () => void;
    onSearchTag: (tag: string) => void;
    onToggleFavorite: (post: Post) => void;
    onDownload: (post: Post) => void;
    onCommentBodyChange: (body: string) => void;
    onSubmitComment: (post: Post) => void;
    onRefreshComments: (postId: number) => void;
    onOpenAccount: () => void;
    onUpdateTags: (post: Post, tagStringDiff: string, editReason: string) => Promise<void>;
    onHideComment: (commentId: number) => void;
  };

  let {
    viewer,
    imageOnly,
    username,
    favoritePending,
    downloadPending,
    downloadStatus,
    comments,
    onClose,
    onToggleImageOnly,
    onSearchTag,
    onToggleFavorite,
    onDownload,
    onCommentBodyChange,
    onSubmitComment,
    onRefreshComments,
    onOpenAccount,
    onUpdateTags,
    onHideComment,
  }: Props = $props();

  let videoElement = $state<HTMLVideoElement | undefined>();
  let imageOnlyVideoElement = $state<HTMLVideoElement | undefined>();
  let videoFrameElement = $state<HTMLDivElement | undefined>();
  let imageOnlyVideoFrameElement = $state<HTMLDivElement | undefined>();
  let appVideoFullscreen = $state(false);
  let showVideoMenu = $state(false);
  let showVideoControls = $state(true);
  let videoControlsTimer: number | undefined;
  let videoUi = $state({
    currentTime: 0,
    duration: 0,
    paused: true,
    muted: false,
    volume: 1,
  });
  let videoPlayback = $state({
    postId: 0,
    currentTime: 0,
    paused: true,
  });

  onMount(() => {
    const onVideoKeydown = (event: KeyboardEvent) => {
      if (!isVideoPost(viewer.post) || !viewer.dataUrl) return;
      if (event.key === "Escape" && appVideoFullscreen) {
        event.preventDefault();
        event.stopImmediatePropagation();
        void exitVideoFullscreen(activeVideoElement());
        return;
      }
      if (event.key !== " " && event.key !== "Spacebar") return;
      if (isTextInput(event.target)) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      void toggleVideoPlayback(activeVideoElement());
    };
    window.addEventListener("keydown", onVideoKeydown, { capture: true });
    return () => {
      window.removeEventListener("keydown", onVideoKeydown, { capture: true });
      window.clearTimeout(videoControlsTimer);
    };
  });

  function activeVideoElement() {
    return imageOnly ? imageOnlyVideoElement : videoElement;
  }

  function isTextInput(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return (
      target.isContentEditable ||
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT"
    );
  }

  function saveVideoPlayback(target: HTMLVideoElement | undefined) {
    if (!target || !Number.isFinite(target.currentTime)) return;
    const duration = Number.isFinite(target.duration) ? target.duration : videoUi.duration;
    const currentTime =
      Number.isFinite(duration) && duration > 0 && duration - target.currentTime < 0.25
        ? duration
        : target.currentTime;
    videoUi = {
      ...videoUi,
      currentTime,
      duration,
      paused: target.paused,
      muted: target.muted,
      volume: target.volume,
    };
    videoPlayback = {
      postId: viewer.post.id,
      currentTime,
      paused: target.paused,
    };
    if (settingsStore.current.playback.remember_volume) {
      playbackMemory.volume = target.volume;
      playbackMemory.muted = target.muted;
    }
  }

  function restoreVideoPlayback(target: HTMLVideoElement | undefined) {
    if (!target || videoPlayback.postId !== viewer.post.id) return;
    const time = videoPlayback.currentTime;
    if (Number.isFinite(time) && Math.abs(target.currentTime - time) > 0.35) {
      target.currentTime = time;
    }
    if (!videoPlayback.paused) {
      void target.play().catch(() => {});
    }
  }

  function syncVideoUi(target: HTMLVideoElement | undefined) {
    if (!target) return;
    videoUi = {
      currentTime: Number.isFinite(target.currentTime) ? target.currentTime : 0,
      duration: Number.isFinite(target.duration) ? target.duration : 0,
      paused: target.paused,
      muted: target.muted,
      volume: target.volume,
    };
  }

  function applyRememberedAudio(target: HTMLVideoElement | undefined) {
    if (!target) return;
    if (!settingsStore.current.playback.remember_volume) return;
    const volume = Math.min(1, Math.max(0, playbackMemory.volume));
    if (Number.isFinite(volume) && Math.abs(target.volume - volume) > 0.001) {
      target.volume = volume;
    }
    if (target.muted !== playbackMemory.muted) {
      target.muted = playbackMemory.muted;
    }
  }

  async function toggleVideoPlayback(target: HTMLVideoElement | undefined) {
    if (!target) return;
    revealVideoControls();
    if (target.paused) {
      await target.play().catch(() => {});
    } else {
      target.pause();
    }
    saveVideoPlayback(target);
  }

  function setVideoRate(target: HTMLVideoElement | undefined, rate: number) {
    if (!target) return;
    revealVideoControls();
    target.playbackRate = rate;
    showVideoMenu = false;
  }

  function copyVideoUrl() {
    const url = viewer.post.file?.url || "";
    if (url) {
      void navigator.clipboard?.writeText(url).catch(() => {});
    }
    showVideoMenu = false;
  }

  function seekVideo(target: HTMLVideoElement | undefined, value: string) {
    if (!target) return;
    revealVideoControls();
    const next = Number(value);
    if (!Number.isFinite(next)) return;
    target.currentTime = next;
    saveVideoPlayback(target);
  }

  function toggleVideoMute(target: HTMLVideoElement | undefined) {
    if (!target) return;
    revealVideoControls();
    target.muted = !target.muted;
    saveVideoPlayback(target);
  }

  function setVideoVolume(target: HTMLVideoElement | undefined, value: string) {
    if (!target) return;
    revealVideoControls();
    const next = Number(value);
    if (!Number.isFinite(next)) return;
    target.volume = Math.min(1, Math.max(0, next));
    target.muted = target.volume === 0;
    saveVideoPlayback(target);
  }

  async function toggleVideoFullscreen(
    target: HTMLVideoElement | undefined,
    frame: HTMLDivElement | undefined,
  ) {
    if (!target || !frame) return;
    revealVideoControls();
    saveVideoPlayback(target);

    if (appVideoFullscreen) {
      await exitVideoFullscreen(target);
      return;
    }

    showVideoMenu = false;
    appVideoFullscreen = true;
    await setWindowFullscreen(true).catch(() => {});
    window.setTimeout(() => restoreVideoPlayback(target), 80);
  }

  async function exitVideoFullscreen(target: HTMLVideoElement | undefined) {
    showVideoMenu = false;
    appVideoFullscreen = false;
    await setWindowFullscreen(false).catch(() => {});
    window.setTimeout(() => restoreVideoPlayback(target), 80);
  }

  function revealVideoControls() {
    showVideoControls = true;
    window.clearTimeout(videoControlsTimer);
    const target = activeVideoElement();
    if (!target || target.paused || showVideoMenu) return;
    videoControlsTimer = window.setTimeout(() => {
      if (!showVideoMenu) {
        showVideoControls = false;
      }
    }, 1800);
  }

  function hideVideoControls() {
    window.clearTimeout(videoControlsTimer);
    if (!showVideoMenu) {
      showVideoControls = false;
    }
  }

  function toggleVideoMenu() {
    showVideoMenu = !showVideoMenu;
    showVideoControls = true;
  }
</script>

{#if imageOnly}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 bg-room-floor" onclick={onToggleImageOnly}>
    {#if viewer.loading}
      <span
        class="absolute left-1/2 top-1/2 size-4 -translate-x-1/2 -translate-y-1/2 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
        aria-hidden="true"
      ></span>
    {:else if viewer.error}
      <div class="absolute left-1/2 top-1/2 max-w-lg -translate-x-1/2 -translate-y-1/2 text-center font-mono text-[11px] leading-relaxed text-room-fav">
        {viewer.error}
      </div>
    {:else if viewer.dataUrl && isVideoPost(viewer.post)}
      <!-- svelte-ignore a11y_media_has_caption -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        bind:this={imageOnlyVideoFrameElement}
        class="{appVideoFullscreen ? 'fixed inset-0 z-[70] bg-room-floor' : 'absolute inset-4'}"
        onclick={(event) => event.stopPropagation()}
        onmousemove={revealVideoControls}
        onmouseenter={revealVideoControls}
        onmouseleave={hideVideoControls}
      >
        <video
          bind:this={imageOnlyVideoElement}
          class="h-full w-full object-contain"
          src={viewer.dataUrl}
          autoplay={settingsStore.current.playback.autoplay}
          loop
          onclick={(event) => event.stopPropagation()}
          ontimeupdate={() => saveVideoPlayback(imageOnlyVideoElement)}
          onpause={() => saveVideoPlayback(imageOnlyVideoElement)}
          onplay={() => {
            saveVideoPlayback(imageOnlyVideoElement);
            revealVideoControls();
          }}
          onvolumechange={() => syncVideoUi(imageOnlyVideoElement)}
          onloadedmetadata={() => {
            applyRememberedAudio(imageOnlyVideoElement);
            syncVideoUi(imageOnlyVideoElement);
            restoreVideoPlayback(imageOnlyVideoElement);
          }}
        ></video>
        <VideoControls
          target={imageOnlyVideoElement}
          frame={imageOnlyVideoFrameElement}
          {videoUi}
          showControls={showVideoControls}
          showMenu={showVideoMenu}
          onReveal={revealVideoControls}
          onTogglePlayback={toggleVideoPlayback}
          onSeek={seekVideo}
          onToggleMute={toggleVideoMute}
          onSetVolume={setVideoVolume}
          onToggleFullscreen={toggleVideoFullscreen}
          onToggleMenu={toggleVideoMenu}
          onSetRate={setVideoRate}
          onCopyUrl={copyVideoUrl}
        />
      </div>
    {:else if viewer.dataUrl}
      <img
        class="absolute inset-4 h-[calc(100%-2rem)] w-[calc(100%-2rem)] object-contain"
        src={viewer.dataUrl}
        alt={postLabel(viewer.post)}
        draggable="false"
      />
    {/if}
  </div>
{:else}
  <div class="fixed inset-0 z-40 grid grid-rows-[42px_minmax(0,1fr)] bg-room-floor">
    <div class="flex items-center justify-between border-b border-room-line bg-room-panel px-4">
      <div class="min-w-0">
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          original
        </div>
        <div class="truncate font-mono text-[12px] tabular-nums text-room-text">
          #{viewer.post.id}
          <span class="text-room-text-low">
            {(viewer.post.file?.ext || "").toUpperCase()} {dimsLabel(viewer.post)}
          </span>
        </div>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="flex size-8 items-center justify-center rounded-[3px] border border-room-line bg-room-panel text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text"
        aria-label="Close original"
      >
        <svg
          class="size-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M18 6 6 18" />
          <path d="m6 6 12 12" />
        </svg>
      </button>
    </div>

    <div class="grid min-h-0 grid-cols-[300px_minmax(0,1fr)]">
      <OriginalPostSidebar
        post={viewer.post}
        {username}
        {onSearchTag}
        {onOpenAccount}
        {onUpdateTags}
      />

      <div class="grid min-h-0 grid-rows-[minmax(0,1fr)_minmax(220px,34vh)]">
        <div class="relative min-h-0 overflow-hidden">
          {#if viewer.loading}
            <span
              class="absolute left-1/2 top-1/2 size-4 -translate-x-1/2 -translate-y-1/2 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
              aria-hidden="true"
            ></span>
          {:else if viewer.error}
            <div class="absolute left-1/2 top-1/2 max-w-lg -translate-x-1/2 -translate-y-1/2 text-center font-mono text-[11px] leading-relaxed text-room-fav">
              {viewer.error}
            </div>
          {:else if viewer.dataUrl && isVideoPost(viewer.post)}
            <!-- svelte-ignore a11y_media_has_caption -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              bind:this={videoFrameElement}
              class="{appVideoFullscreen ? 'fixed inset-0 z-[70] bg-room-floor' : 'absolute inset-4'}"
              onmousemove={revealVideoControls}
              onmouseenter={revealVideoControls}
              onmouseleave={hideVideoControls}
            >
              <video
                bind:this={videoElement}
                class="h-full w-full object-contain"
                src={viewer.dataUrl}
                autoplay={settingsStore.current.playback.autoplay}
                loop
                ontimeupdate={() => saveVideoPlayback(videoElement)}
                onpause={() => saveVideoPlayback(videoElement)}
                onplay={() => {
                  saveVideoPlayback(videoElement);
                  revealVideoControls();
                }}
                onvolumechange={() => syncVideoUi(videoElement)}
                onloadedmetadata={() => {
                  applyRememberedAudio(videoElement);
                  syncVideoUi(videoElement);
                  restoreVideoPlayback(videoElement);
                }}
              ></video>
              <VideoControls
                target={videoElement}
                frame={videoFrameElement}
                {videoUi}
                showControls={showVideoControls}
                showMenu={showVideoMenu}
                onReveal={revealVideoControls}
                onTogglePlayback={toggleVideoPlayback}
                onSeek={seekVideo}
                onToggleMute={toggleVideoMute}
                onSetVolume={setVideoVolume}
                onToggleFullscreen={toggleVideoFullscreen}
                onToggleMenu={toggleVideoMenu}
                onSetRate={setVideoRate}
                onCopyUrl={copyVideoUrl}
              />
            </div>
          {:else if viewer.dataUrl}
            <button
              type="button"
              onclick={onToggleImageOnly}
              class="absolute inset-4 flex cursor-zoom-in items-center justify-center bg-transparent p-0"
              aria-label="Open image only view"
            >
              <img
                class="h-full w-full object-contain"
                src={viewer.dataUrl}
                alt={postLabel(viewer.post)}
                draggable="false"
              />
            </button>
          {/if}
        </div>

        <CommentsPanel
          post={viewer.post}
          {username}
          {favoritePending}
          {downloadPending}
          {downloadStatus}
          {comments}
          {onToggleFavorite}
          {onDownload}
          {onCommentBodyChange}
          {onSubmitComment}
          {onRefreshComments}
          {onOpenAccount}
          {onHideComment}
        />
      </div>
    </div>
  </div>
{/if}
