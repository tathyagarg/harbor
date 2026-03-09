<script lang="ts">
  import TTFTable from "../TTFTable.svelte";

  let tables = [
    {
      name: "glyf",
      description: "Contains the actual glyph data for each character.",
    },
    {
      name: "cmap",
      description: "Maps Unicode code points to glyph indices in the font.",
    },
    {
      name: "hmtx",
      description:
        "Contains horizontal metrics for each glyph, such as advance width",
    },
    {
      name: "head",
      description:
        "Contains global information about the font, such as version and bounding box.",
    },
    {
      name: "maxp",
      description:
        "Contains information about the maximum number of glyphs and points in the font.",
    },
    {
      name: "And more..",
      description:
        "There are many more tables in a TTF file, each serving a specific purpose.",
    },
  ];
</script>

<div class="flex flex-row gap-8 items-center justify-center w-full h-full">
  <div class="">
    <div
      class="aspect-[5/4] bg-(--text) flex items-center justify-center rounded-lg px-4"
    >
      <span class="text-black text-center text-8xl font-mono">Aa</span>
    </div>
    <p class="w-full text-center my-2">Mono.ttf</p>
  </div>
  <span class="text-6xl text-(--text)">→</span>
  <div class="w-[40%]">
    <h1 class="text-2xl text-center my-2 font-bold">Font Tables</h1>
    <hr class="my-2" />
    <div class="grid grid-cols-2 gap-4">
      {#each tables as table}
        <TTFTable name={table.name} description={table.description} />
      {/each}
    </div>
  </div>
</div>
