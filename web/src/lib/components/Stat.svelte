<script lang="ts">
  export let title: string;
  export let value: string;
</script>

<div
  class="flex flex-col items-center gap-2 w-full py-4 border-2 border-(white/20) rounded-lg"
>
  <p class="text-5xl font-bold">{value}</p>
  <h3 class="text-(white/80) text-sm">{title}</h3>
</div>
