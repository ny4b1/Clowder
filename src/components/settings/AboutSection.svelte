<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { updateStore } from "../../lib/update-store.svelte";

  // The app version never changes at runtime, so cache the result across
  // remounts (each settings-tab change unmounts this component).
  let cachedVersion: string | null = null;

  let appVersion = $state(cachedVersion ?? "");

  onMount(() => {
    if (cachedVersion !== null) return;
    void getVersion()
      .then((value) => {
        cachedVersion = value;
        appVersion = value;
      })
      .catch(() => {
        cachedVersion = "";
      });
  });
</script>

<div class="space-y-4 px-5 py-4">
  <div>
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      version
    </div>
    <div class="mt-1 flex items-center gap-3">
      <span class="font-mono text-[13px] tabular-nums text-room-text">
        {appVersion || "—"}
      </span>
      <span class="font-mono text-[10.5px] text-room-text-low">
        {#if updateStore.status === "checking"}
          checking…
        {:else if updateStore.status === "available" && updateStore.available}
          v{updateStore.available.version} available
        {:else if updateStore.status === "downloading"}
          downloading…
        {:else if updateStore.status === "ready"}
          installed — restart to apply
        {:else if updateStore.status === "error"}
          <span class="text-room-fav">{updateStore.error}</span>
        {:else}
          up to date
        {/if}
      </span>
    </div>
    <div class="mt-2 flex items-center gap-2">
      <button
        type="button"
        onclick={() => void updateStore.check()}
        disabled={updateStore.status === "checking" || updateStore.status === "downloading"}
        class="h-7 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
      >
        check now
      </button>
      {#if updateStore.status === "available"}
        <button
          type="button"
          onclick={() => void updateStore.install()}
          class="h-7 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20"
        >
          install
        </button>
      {/if}
    </div>
  </div>
  <div>
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      source
    </div>
    <button
      type="button"
      onclick={() => {
        void openUrl("https://github.com/nyattic/Clowder").catch(() => {});
      }}
      class="mt-1 inline-flex items-center gap-1.5 font-mono text-[12px] text-room-text-mid transition-colors duration-150 hover:text-room-accent"
    >
      github.com/nyattic/Clowder
      <svg
        class="size-3"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M7 17 17 7" />
        <path d="M7 7h10v10" />
      </svg>
    </button>
  </div>
  <div>
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      license
    </div>
    <div class="mt-1 font-mono text-[12px] text-room-text-mid">GPL-3.0</div>
  </div>
  <p class="pt-2 text-[11.5px] leading-relaxed text-room-text-low">
    Not affiliated with or endorsed by e621 or e6ai.
  </p>
</div>
