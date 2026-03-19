<script lang="ts">
  export let name: string;
  export let description: string;
</script>

<div
  class="flex flex-col gap-2 bg-celadon/25 rounded-lg p-4 border border-celadon"
>
  <h3 class="text-lg font-bold text-center font-mono">{name}</h3>
  <p class="text-xs">{description}</p>
</div>
