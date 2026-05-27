<script lang="ts">
  import type { AppearanceSettings, MotionPreference, Theme } from "../../lib/types";

  type Props = {
    appearance: AppearanceSettings;
    saving: boolean;
  };

  let { appearance = $bindable(), saving }: Props = $props();

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "system", label: "System" },
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
  ];

  const motionOptions: { value: MotionPreference; label: string; description: string }[] = [
    { value: "system", label: "System", description: "respect prefers-reduced-motion" },
    { value: "always", label: "Always animate", description: "ignore reduced motion request" },
    { value: "never", label: "Never animate", description: "disable all transitions" },
  ];

  const tilePresets = [
    { value: 144, label: "Compact" },
    { value: 176, label: "Comfortable" },
    { value: 220, label: "Spacious" },
  ];
</script>

<div class="space-y-5 px-5 py-4">
  <div class="space-y-2">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      theme
    </div>
    <div class="flex flex-wrap gap-1.5">
      {#each themeOptions as option (option.value)}
        {@const isActive = appearance.theme === option.value}
        <button
          type="button"
          onclick={() => (appearance.theme = option.value)}
          disabled={saving}
          class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
            ? 'border-room-accent text-room-accent'
            : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
        >
          {option.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="space-y-2 border-t border-room-line pt-4">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      motion
    </div>
    <div class="space-y-1">
      {#each motionOptions as option (option.value)}
        {@const isActive = appearance.motion === option.value}
        <label
          class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {isActive
            ? 'border-room-accent'
            : 'border-room-line hover:border-room-line-strong'}"
        >
          <input
            type="radio"
            name="motion"
            value={option.value}
            checked={isActive}
            disabled={saving}
            onchange={() => (appearance.motion = option.value)}
            class="mt-1 accent-room-accent"
          />
          <span class="min-w-0 flex-1">
            <span
              class="block text-[12.5px] {isActive ? 'text-room-accent' : 'text-room-text'}"
            >
              {option.label}
            </span>
            <span
              class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
            >
              {option.description}
            </span>
          </span>
        </label>
      {/each}
    </div>
  </div>

  <div class="space-y-2 border-t border-room-line pt-4">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      grid tile size
    </div>
    <div class="flex flex-wrap gap-1.5">
      {#each tilePresets as preset (preset.value)}
        {@const isActive = appearance.grid_min_tile_px === preset.value}
        <button
          type="button"
          onclick={() => (appearance.grid_min_tile_px = preset.value)}
          disabled={saving}
          class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
            ? 'border-room-accent text-room-accent'
            : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
        >
          {preset.label}
        </button>
      {/each}
    </div>
    <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
      current minimum: {appearance.grid_min_tile_px}px
    </p>
  </div>
</div>
