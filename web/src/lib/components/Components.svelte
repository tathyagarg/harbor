<script lang="ts">
  export let icon: string;
  export let title: string;
  export let description: string;

  export let loc: string;
</script>

<div
  class="flex flex-col gap-4 bg-(--baltic-blue)/50 p-6 rounded-lg shadow-lg border-2 border-(--celadon)/15
  "
>
  <div class="flex justify-between">
    <span class="text-2xl text-(--celadon)">{icon}</span>
    <p
      class="text-(white/50) text-sm border-2
      border-white/50 px-4 py-1 rounded-full
      h-fit
      "
    >
      ~{loc} lines
    </p>
  </div>
  <h3 class="text-xl font-semibold">{title}</h3>
  <p class="text-sm">{description}</p>
</div>
