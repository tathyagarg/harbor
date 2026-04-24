<script lang="ts">
  // @ts-nocheck

  import * as d3 from "d3";
  import { onMount } from "svelte";

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

  onMount(async () => {
    const lineCount = await fetch(
      "https://raw.githubusercontent.com/tathyagarg/harbor/refs/heads/main/.github/lines.json",
    )
      .then((response) => response.json())
      .then((data) => data.lines);

    console.log(JSON.stringify(lineCount, null, 2));
    console.log(makeTreeFromLineData(lineCount));

    const data = makeTreeFromLineData(lineCount);
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
  });
</script>

<div class="w-full h-full relative">
  <svg bind:this={svg} class="absolute top-0 left-0 w-full h-full"></svg>
</div>
