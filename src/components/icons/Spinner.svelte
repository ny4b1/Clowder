<script lang="ts">
  type Props = {
    class?: string;
  };

  let {
    class: className = "size-3 border border-room-line-strong border-t-room-accent",
  }: Props = $props();
</script>

<span class="inline-block animate-spin rounded-full {className}" aria-hidden="true"></span>
