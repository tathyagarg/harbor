<script lang="ts">
  let { name } = $props();
</script>

<div
  class="w-full h-full rounded-2xl overflow-hidden border-1 border-emphasis-1/25 shadow-lg shadow-white/20"
>
  <button
    onclick={() => (window.location.href = `/core/${name}`)}
    class="w-full h-full bg-bg py-12 cursor-pointer"
  >
    <span class="uppercase font-mono text-2xl">
      {name}
    </span>
  </button>
</div>
