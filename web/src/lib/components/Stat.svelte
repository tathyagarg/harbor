<script lang="ts">
  import Countup from "svelte-countup";

  let {
    value,
    title,
    suffix,
  }: { value: number; title: string; suffix: string | null } = $props();

  let duration = 2000;
</script>

<div class="flex flex-col items-center gap-1">
  <span class="text-5xl font-bold text-emphasis-2 font-heading">
    <Countup {value} {duration} />{#if suffix}<span class="text-3xl"
        >{suffix}</span
      >{/if}
  </span>
  <span class="text-sm text-gray-500 uppercase text-center">{title}</span>
</div>
