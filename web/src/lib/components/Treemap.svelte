<script lang="ts">
  // @ts-nocheck

  import * as d3 from "d3";
  import { onMount } from "svelte";
  import Heading from "./Heading.svelte";

  let { lines } = $props();

  let svg: SVGElement;

  function makeTreeFromLineData(data: any, dirName: string = "root"): any {
    if (dirName === "__total__") return null;

    let childrenData = [];

    for (const [dir, children] of Object.entries(data)) {
      if (dir === "__total__") continue;

      if (typeof children === "number") {
        childrenData.push({ name: dir, value: children });
      } else {
        childrenData.push(makeTreeFromLineData(children, dir));
      }
    }

    return {
      name: dirName,
      children: childrenData,
    };
  }

  const width = 1800;
  const height = 1000;

  let dataMap = $state({});

  onMount(async () => {
    const data = makeTreeFromLineData(lines);
    const color = d3.scaleOrdinal(
      data.children.map((d) => d.name),
      d3.schemeTableau10,
    );

    const root = d3
      .hierarchy(data)
      .sum((d) => d.value || 0)
      .sort((a, b) => b.value - a.value);

    // treemap layout
    d3.treemap().size([width, height]).round(true).padding(2)(root);

    const nodes = root.leaves();

    const g = d3
      .select(svg)
      .attr("viewBox", `0 400 ${width} ${height / 2}`)
      .attr("style", "font: 15px sans-serif;");

    const cell = g
      .selectAll("g")
      .data(nodes)
      .enter()
      .append("g")
      .attr("transform", (d) => `translate(${d.x0},${d.y0})`);

    cell
      .append("rect")
      .attr("width", (d) => d.x1 - d.x0)
      .attr("height", (d) => d.y1 - d.y0)
      .attr("fill", (d) => {
        while (d.depth > 4) d = d.parent;

        dataMap[d.data.name] = {
          color: color(d.data.name),
          size: d.value,
        };
        return color(d.data.name);
      });

    cell.append("title").text(
      (d) =>
        `${d
          .ancestors()
          .map((d) => d.data.name)
          .reverse()
          .join("/")}\n${d.value} lines`,
    );

    cell
      .append("text")
      .selectAll("tspan")
      .data((d) =>
        d.data.name.split(/(?=[A-Z][^A-Z])/g).concat(`${d.value} lines`),
      )
      .join("tspan")
      .attr("x", 3)
      .attr("y", (_, i) => 15 + i * 15)
      .attr("fill-opacity", (_, i, nodes) => (i === nodes.length - 1 ? 0.7 : 1))
      .text((d) => d);

    dataMap = Object.entries(dataMap)
      .sort((a, b) => b[1].size - a[1].size)
      .slice(0, 8);
  });
</script>

<div class="w-full h-full relative">
  <svg bind:this={svg} class="w-full h-1/2"></svg>

  <div class="w-full h-1/2 flex flex-col items-start justify-start gap-4 p-4">
    <div class="border-b-1 border-emphasis-1/25 w-full">
      <Heading text="Color Map (Legend)" />
    </div>

    <div class="flex-1 grid grid-cols-2 gap-8">
      {#each dataMap as [name, { color, size }], i}
        {@const colStart = Math.floor(i / 5) + 1}
        {@const rowStart = (i % 5) + 1}

        <div
          class="flex items-center gap-2"
          style="grid-row: {rowStart}; grid-column: {colStart};"
        >
          <span>{i + 1}.</span>
          <div
            class="w-4 h-4 rounded-sm"
            style="background-color: {color};"
          ></div>
          <span class="font-mono">{name} ({size} lines)</span>
        </div>
      {/each}
    </div>
  </div>
</div>
