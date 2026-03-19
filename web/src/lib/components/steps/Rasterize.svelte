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

  let glyphData = {
    name: "A",
    unicode: "U+0041",
    advanceWidth: 600,
    boundingBox: {
      xMin: 0,
      yMin: -200,
      xMax: 600,
      yMax: 700,
    },
    contours: [
      [
        { x: 0, y: 0 },
        { x: 300, y: 700 },
        { x: 600, y: 0 },
      ],
      [
        { x: 200, y: 200 },
        { x: 300, y: 500 },
        { x: 400, y: 200 },
      ],
    ],
  };
</script>

<div class="flex flex-row gap-8 items-center justify-center w-full h-full">
  <div class="w-[40%]">
    <h1 class="text-2xl text-center my-2 font-bold">Font Tables</h1>
    <hr class="my-2" />
    <div class="grid grid-cols-2 gap-4">
      {#each tables as table}
        <TTFTable name={table.name} description={table.description} />
      {/each}
    </div>
  </div>
  <span class="text-6xl text-(--text)">→</span>
  <div class="w-[40%]">
    <h1 class="text-2xl text-center my-2 font-bold">Glyph Data</h1>
    <hr class="my-2" />
    <div class="bg-celadon text-black p-4 rounded-lg">
      <h2 class="text-xl font-bold">{glyphData.name}</h2>
      <p>Unicode: {glyphData.unicode}</p>
      <p>Advance Width: {glyphData.advanceWidth}</p>
      <p>
        Bounding Box: ({glyphData.boundingBox.xMin}, {glyphData.boundingBox
          .yMin}) to ({glyphData.boundingBox.xMax}, {glyphData.boundingBox
          .yMax})
      </p>
      <p>Contours:</p>
      <ol class="list-decimal pl-6">
        {#each glyphData.contours as contour}
          <li>
            <ul class="list-disc pl-6">
              {#each contour as point}
                <li>({point.x}, {point.y})</li>
              {/each}
            </ul>
          </li>
        {/each}
      </ol>
    </div>
  </div>
</div>
