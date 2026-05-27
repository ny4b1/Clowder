<script lang="ts">
  import CloseIcon from "./icons/CloseIcon.svelte";
  import { toastStore } from "../lib/toast-store.svelte";
</script>

<div
  class="pointer-events-none fixed bottom-4 right-4 z-[80] flex max-w-sm flex-col gap-2"
  role="region"
  aria-live="polite"
  aria-label="notifications"
>
  {#each toastStore.items as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-start gap-3 rounded-[3px] border bg-room-panel-hi px-3 py-2 shadow-[0_12px_32px_rgba(0,0,0,0.45)] {toast.kind ===
      'error'
        ? 'border-room-fav'
        : toast.kind === 'success'
          ? 'border-room-accent'
          : 'border-room-line-strong'}"
    >
      <span
        class="mt-1 size-1.5 shrink-0 rounded-full {toast.kind === 'error'
          ? 'bg-room-fav'
          : toast.kind === 'success'
            ? 'bg-room-accent'
            : 'bg-room-text-mid'}"
        aria-hidden="true"
      ></span>
      <p class="min-w-0 flex-1 break-words font-mono text-[11.5px] leading-relaxed text-room-text">
        {toast.message}
      </p>
      <button
        type="button"
        onclick={() => toastStore.dismiss(toast.id)}
        class="-mr-1 -mt-1 flex size-6 shrink-0 items-center justify-center rounded-[3px] text-room-text-low transition-colors duration-150 hover:text-room-text"
        aria-label="Dismiss notification"
      >
        <CloseIcon class="size-3" />
      </button>
    </div>
  {/each}
</div>
