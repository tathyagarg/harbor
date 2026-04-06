<script lang="ts">
  let data: { color: string; index: number } = $props();
</script>

<span
  class="h-3 w-[90%] absolute left-2 line"
  style={`top: calc(var(--spacing)*${data.index * 4}); background: ${data.color};`}
></span>
