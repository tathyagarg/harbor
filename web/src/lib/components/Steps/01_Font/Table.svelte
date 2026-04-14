<script lang="ts">
  import Icon from "@iconify/svelte";

  let data: { name: string; color: string } = $props();
</script>

<div
  class="text-3xl font-code flex items-center justify-center h-full
  rounded-lg border-1 px-12 py-2 fonttable"
  style={`background: rgba(from ${data.color} r g b / 25%); border-color: ${data.color}; color: ${data.color};`}
  id={data.name}
>
  <Icon icon="mdi:table" class="text-3xl mr-2" style={`color: ${data.color}`} />
  {data.name}
</div>
