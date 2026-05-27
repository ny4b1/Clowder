<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { errMsg } from "../../lib/errors";
  import { vpnStore } from "../../lib/vpn-store.svelte";
  import Spinner from "../icons/Spinner.svelte";

  onMount(() => {
    if (!vpnStore.loaded) {
      void vpnStore.refresh();
    }
  });

  let status = $derived(vpnStore.status);
  let configured = $derived(status.configured);
  let enabled = $derived(status.enabled);

  let stateLabel = $derived.by(() => {
    if (!configured) return "no config";
    if (vpnStore.busy) return "working";
    return enabled ? "connected" : "disconnected";
  });

  let stateClass = $derived.by(() => {
    if (!configured) return "text-room-text-low";
    if (vpnStore.busy) return "text-room-text-mid";
    return enabled ? "text-room-accent" : "text-room-text-mid";
  });

  async function pickConfig() {
    try {
      const selected = await openDialog({
        directory: false,
        multiple: false,
        title: "Select WireGuard config",
        filters: [{ name: "WireGuard config", extensions: ["conf"] }],
      });
      if (typeof selected === "string" && selected) {
        await vpnStore.import(selected);
      }
    } catch (error) {
      vpnStore.error = errMsg(error);
    }
  }

  async function toggle() {
    if (vpnStore.busy) return;
    if (enabled) {
      await vpnStore.disable();
    } else {
      await vpnStore.enable();
    }
  }

  async function remove() {
    if (vpnStore.busy) return;
    await vpnStore.clear();
  }
</script>

<div class="space-y-5 px-5 py-4">
  <div class="space-y-2">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      tunnel
    </div>
    <div class="flex items-center gap-2">
      <span
        class="inline-block size-2 rounded-full {enabled && configured
          ? 'bg-room-accent'
          : configured
            ? 'bg-room-text-mid'
            : 'bg-room-line-strong'}"
        aria-hidden="true"
      ></span>
      <span class="font-mono text-[11.5px] {stateClass}">
        {stateLabel}
      </span>
      {#if vpnStore.busy}
        <Spinner class="size-2.5 border border-room-text-mid/40 border-t-room-text-mid" />
      {/if}
    </div>
    {#if configured && status.endpoint}
      <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
        endpoint: <span class="text-room-text-mid">{status.endpoint}</span>
      </p>
    {/if}
    {#if enabled && status.proxy_url}
      <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
        proxy: <span class="text-room-text-mid">{status.proxy_url}</span>
      </p>
    {/if}
  </div>

  <div class="space-y-2 border-t border-room-line pt-4">
    {#if configured}
      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onclick={toggle}
          disabled={vpnStore.busy}
          class="h-8 rounded-[3px] border px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {enabled
            ? 'border-room-line bg-room-panel text-room-text-mid hover:border-room-fav hover:text-room-fav'
            : 'border-room-accent bg-room-accent/10 text-room-accent hover:bg-room-accent/20'}"
        >
          {enabled ? "disconnect" : "connect"}
        </button>
        <button
          type="button"
          onclick={pickConfig}
          disabled={vpnStore.busy}
          class="h-8 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
        >
          replace config
        </button>
        <button
          type="button"
          onclick={remove}
          disabled={vpnStore.busy}
          class="h-8 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-fav hover:text-room-fav disabled:opacity-50"
        >
          remove
        </button>
      </div>
    {:else}
      <button
        type="button"
        onclick={pickConfig}
        disabled={vpnStore.busy}
        class="flex h-8 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-50"
      >
        {#if vpnStore.busy}
          <Spinner class="size-2.5 border border-room-accent/40 border-t-room-accent" />
        {/if}
        import wireguard config
      </button>
    {/if}
    {#if vpnStore.error}
      <p class="font-mono text-[10.5px] leading-relaxed text-room-fav">
        {vpnStore.error}
      </p>
    {/if}
  </div>

  <div class="space-y-1 border-t border-room-line pt-4">
    <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
      Only this app's traffic goes through the tunnel. Export a WireGuard config
      from Mullvad or Proton, then import the .conf file here.
    </p>
    <p class="font-mono text-[10px] leading-relaxed text-room-text-low">
      Private key is stored in your system keychain. DNS is resolved through
      the tunnel to prevent leaks.
    </p>
  </div>
</div>
