<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { errMsg } from "../../lib/errors";
  import { vpnStore } from "../../lib/vpn-store.svelte";
  import Spinner from "../icons/Spinner.svelte";

  let account = $state("");
  let showAdvanced = $state(false);
  let selectedCountry = $state("");
  let selectedCity = $state("");

  onMount(async () => {
    if (!vpnStore.loaded) {
      await vpnStore.refresh();
    }
    if (vpnStore.status.provider === "mullvad") {
      void vpnStore.loadLocations();
    }
  });

  let status = $derived(vpnStore.status);
  let provider = $derived(status.provider);
  let configured = $derived(status.configured);
  let enabled = $derived(status.enabled);
  let isMullvad = $derived(provider === "mullvad");
  let isManual = $derived(provider === "manual");

  let countries = $derived(vpnStore.locations);
  let cities = $derived(
    countries.find((c) => c.code === selectedCountry)?.cities ?? [],
  );

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

  $effect(() => {
    if (isMullvad && countries.length > 0 && selectedCity === "") {
      const countryCode = status.country_code ?? countries[0].code;
      const list = countries.find((c) => c.code === countryCode)?.cities ?? [];
      selectedCountry = countryCode;
      selectedCity = status.city_code ?? list[0]?.code ?? "";
    }
  });

  const selectClass =
    "h-8 min-w-0 flex-1 rounded-[3px] border border-room-line bg-room-panel px-2 font-mono text-[11px] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong focus:border-room-accent focus:outline-none disabled:opacity-50";

  async function signIn() {
    const trimmed = account.trim();
    if (!trimmed || vpnStore.busy) return;
    const ok = await vpnStore.signInMullvad(trimmed);
    if (ok) {
      account = "";
    }
  }

  async function applyCountry() {
    const list = countries.find((c) => c.code === selectedCountry)?.cities ?? [];
    const city = list[0]?.code ?? "";
    selectedCity = city;
    if (city) {
      await vpnStore.selectRelay(city);
    }
  }

  async function applyCity() {
    if (selectedCity) {
      await vpnStore.selectRelay(selectedCity);
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

  async function signOut() {
    if (vpnStore.busy) return;
    selectedCountry = "";
    selectedCity = "";
    await vpnStore.signOutMullvad();
  }

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

  async function removeManual() {
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
    {#if isMullvad && status.city}
      <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
        location: <span class="text-room-text-mid">{status.city}, {status.country}</span>
      </p>
    {:else if configured && status.endpoint}
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

  {#if isMullvad}
    <div class="space-y-3 border-t border-room-line pt-4">
      <div class="space-y-0.5">
        <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
          Mullvad account <span class="text-room-text-mid">{status.account}</span>
        </p>
        {#if status.device}
          <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
            device <span class="text-room-text-mid">{status.device}</span>
          </p>
        {/if}
      </div>
      <div class="space-y-2">
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          server
        </div>
        {#if countries.length === 0}
          <p class="flex items-center gap-1.5 font-mono text-[10.5px] text-room-text-low">
            <Spinner class="size-2.5 border border-room-text-mid/40 border-t-room-text-mid" />
            loading servers…
          </p>
        {:else}
          <div class="flex items-center gap-2">
            <select
              bind:value={selectedCountry}
              onchange={applyCountry}
              disabled={vpnStore.busy}
              class={selectClass}
              aria-label="Country"
            >
              {#each countries as country (country.code)}
                <option value={country.code}>{country.name}</option>
              {/each}
            </select>
            <select
              bind:value={selectedCity}
              onchange={applyCity}
              disabled={vpnStore.busy || cities.length === 0}
              class={selectClass}
              aria-label="City"
            >
              {#each cities as city (city.code)}
                <option value={city.code}>{city.name}</option>
              {/each}
            </select>
          </div>
        {/if}
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onclick={toggle}
          disabled={vpnStore.busy || !configured}
          class="h-8 rounded-[3px] border px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {enabled
            ? 'border-room-line bg-room-panel text-room-text-mid hover:border-room-fav hover:text-room-fav'
            : 'border-room-accent bg-room-accent/10 text-room-accent hover:bg-room-accent/20'}"
        >
          {enabled ? "disconnect" : "connect"}
        </button>
        <button
          type="button"
          onclick={signOut}
          disabled={vpnStore.busy}
          class="h-8 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-fav hover:text-room-fav disabled:opacity-50"
        >
          sign out
        </button>
      </div>
    </div>
  {:else if isManual}
    <div class="space-y-2 border-t border-room-line pt-4">
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
          onclick={removeManual}
          disabled={vpnStore.busy}
          class="h-8 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-fav hover:text-room-fav disabled:opacity-50"
        >
          remove
        </button>
      </div>
    </div>
  {:else}
    <div class="space-y-3 border-t border-room-line pt-4">
      <div class="space-y-2">
        <label
          for="mullvad-account"
          class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
        >
          mullvad account number
        </label>
        <input
          id="mullvad-account"
          bind:value={account}
          onkeydown={(e) => {
            if (e.key === "Enter") signIn();
          }}
          inputmode="numeric"
          autocomplete="off"
          spellcheck="false"
          placeholder="0000 0000 0000 0000"
          disabled={vpnStore.busy}
          class="h-8 w-full rounded-[3px] border border-room-line bg-room-panel px-2.5 font-mono text-[12px] tracking-[0.08em] text-room-text placeholder:text-room-text-low/60 transition-colors duration-150 focus:border-room-accent focus:outline-none disabled:opacity-50"
        />
      </div>
      <button
        type="button"
        onclick={signIn}
        disabled={vpnStore.busy || account.trim() === ""}
        class="flex h-8 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-50"
      >
        {#if vpnStore.busy}
          <Spinner class="size-2.5 border border-room-accent/40 border-t-room-accent" />
        {/if}
        sign in
      </button>

      <div class="pt-1">
        <button
          type="button"
          onclick={() => (showAdvanced = !showAdvanced)}
          class="font-mono text-[10px] uppercase tracking-[0.18em] text-room-text-low transition-colors duration-150 hover:text-room-text-mid"
        >
          {showAdvanced ? "−" : "+"} use a custom wireguard config
        </button>
        {#if showAdvanced}
          <div class="pt-2">
            <button
              type="button"
              onclick={pickConfig}
              disabled={vpnStore.busy}
              class="flex h-8 items-center gap-1.5 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
            >
              import wireguard config
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if vpnStore.error}
    <p class="font-mono text-[10.5px] leading-relaxed text-room-fav">
      {vpnStore.error}
    </p>
  {/if}

  <div class="space-y-1.5 border-t border-room-line pt-4">
    <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
      Only this app's traffic goes through the tunnel. Enter your Mullvad
      account number and pick a server in a country where e621 is reachable.
    </p>
    <p class="font-mono text-[10px] leading-relaxed text-room-fav/90">
      Don't keep a system-wide VPN (e.g. the Mullvad app) connected at the same
      time — stacking two VPNs nests the tunnels and breaks the connection, so
      sign-in and browsing fail. Use one or the other.
    </p>
    <p class="font-mono text-[10px] leading-relaxed text-room-text-low">
      Your account number and key live in your system keychain. DNS is resolved
      through the tunnel to prevent leaks.
    </p>
  </div>
</div>
