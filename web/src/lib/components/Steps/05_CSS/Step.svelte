<script lang="ts">
  import Icon from "@iconify/svelte";
  import { animate, onScroll } from "animejs";
  import { onMount } from "svelte";

  const cssLines = [
    "body {",
    "  margin: 0;",
    "}",
    "h1 {",
    "  font-size: 2rem;",
    "  color: white;",
    "}",
  ];

  const nodeColors: Record<string, string> = {
    root: "var(--color-pretty-blue)",
    rule: "var(--color-emphasis-2)",
    declaration: "var(--color-emphasis-3)",
  };

  const nodes = [
    {
      id: "n0",
      label: "CSSStyleSheet",
      x: 100,
      y: 0,
      parent: null,
      kind: "root",
    },
    {
      id: "n1",
      label: "CSSStyleRule[0]",
      x: 0,
      y: 100,
      parent: "n0",
      kind: "rule",
    },
    {
      id: "n3",
      label: "margin",
      x: 0,
      y: 200,
      parent: "n1",
      kind: "declaration",
    },
    {
      id: "n2",
      label: "CSSStyleRule[1]",
      x: 200,
      y: 100,
      parent: "n0",
      kind: "rule",
    },
    {
      id: "n4",
      label: "font-size",
      x: 125,
      y: 200,
      parent: "n2",
      kind: "declaration",
    },
    {
      id: "n5",
      label: "color",
      x: 250,
      y: 200,
      parent: "n2",
      kind: "declaration",
    },
  ];

  const edges = [
    { from: "n0", to: "n1" },
    { from: "n0", to: "n2" },
    { from: "n1", to: "n3" },
    { from: "n2", to: "n4" },
    { from: "n2", to: "n5" },
  ];

  let lineIndex = $state(0);

  onMount(async () => {
    animate(".node", {
      opacity: 0,
      translateY: 20,
      duration: 0,
    });

    await animate(".node", {
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
    });

    await new Promise((resolve) => setTimeout(resolve, 1000));

    const lineInterval = setInterval(async () => {
      if (lineIndex >= cssLines.length) {
        clearInterval(lineInterval);
        return;
      }

      let codeLine = document.createElement("div");
      codeLine.textContent = cssLines[lineIndex++];

      animate(codeLine, {
        opacity: 0,
        translateX: -20,
        duration: 0,
      });

      code.appendChild(codeLine);

      await animate(codeLine, {
        opacity: 1,
        translateX: 0,
        duration: 300,
        ease: "outQuad",
      });
    }, 350);

    for (const node of nodes) {
      await animate(`#${node.id}`, {
        opacity: 1,
        translateY: 0,
        duration: 500,
        ease: "outQuad",
        onBegin: () => {
          if (node.parent) {
            const edge = document.getElementById(
              `edge-${node.parent}-${node.id}`,
            );
            if (edge) {
              animate(edge, {
                strokeDashoffset: [200, 0],
                duration: 500,
                ease: "linear",
              });
            }
          }
        },
      });
    }
  });

  let code: HTMLElement;
  let nodesParent: SVGSVGElement;
</script>

<div class="w-full h-full flex items-center justify-center px-8 text-subtext">
  <div class="flex-2 h-full flex flex-col justify-center w-full p-2">
    <div class="border-b-1 pb-2 text-xs mb-2 font-mono uppercase">
      CSS Source
    </div>
    <pre
      class="text-white text-xs font-mono text-left w-full border-1 border-emphasis-1/25 rounded-lg p-2 bg-pretty-blue/5"><div
        bind:this={code}></div></pre>
  </div>
  <div class="flex-1 h-full relative">
    <Icon
      icon="mdi:arrow-right-bold"
      class="w-12 h-12 absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
    />
  </div>
  <div class="flex-3 h-full flex flex-col justify-center w-full p-2">
    <div class="border-b-1 pb-2 text-xs mb-2 font-mono uppercase">
      CSSOM Tree
    </div>
    <svg
      bind:this={nodesParent}
      class="w-full h-[70%] bg-emphasis-3/5 rounded-lg p-2 border-1 border-emphasis-1/25"
      viewBox="-10 20 370 200"
      preserveAspectRatio="xMidYMid meet"
    >
      <defs>
        <filter id="root-glow">
          <feDropShadow
            dx="0"
            dy="0"
            stdDeviation="3"
            flood-color="var(--color-pretty-blue)"
          />
        </filter>
        <filter id="rule-glow">
          <feDropShadow
            dx="0"
            dy="0"
            stdDeviation="3"
            flood-color="var(--color-emphasis-2)"
          />
        </filter>
        <filter id="declaration-glow">
          <feDropShadow
            dx="0"
            dy="0"
            stdDeviation="3"
            flood-color="var(--color-emphasis-3)"
          />
        </filter>
      </defs>

      {#each edges as edge}
        {@const from_node = nodes.find((n) => n.id === edge.from) ?? {
          x: 0,
          y: 0,
        }}
        {@const to_node = nodes.find((n) => n.id === edge.to) ?? { x: 0, y: 0 }}

        <line
          x1={from_node.x + 50}
          y1={from_node.y + 40}
          x2={to_node.x + 50}
          y2={to_node.y}
          stroke="var(--color-emphasis-1)"
          stroke-width="2"
          opacity="0.5"
          stroke-dasharray="200"
          stroke-dashoffset="200"
          id={`edge-${edge.from}-${edge.to}`}
        ></line>
      {/each}

      {#each nodes as node}
        {@const nodeColor = nodeColors[node.kind] ?? "var(--color-emphasis-1)"}

        <g id={node.id} class="node">
          <rect
            x={node.x}
            y={node.y}
            width="100"
            height="40"
            opacity="0.5"
            rx="5"
            ry="5"
            fill={`rgba(from ${nodeColor} r g b / 0.25)`}
            stroke={nodeColor}
            stroke-width="2"
            filter={`url(#${node.kind}-glow)`}
          ></rect>

          <text
            x={node.x + 50}
            y={node.y + 20}
            fill={nodeColor}
            font-size="12"
            font-family="Arial"
            dominant-baseline="middle"
            text-anchor="middle"
            font-weight="bold"
          >
            {node.label}
          </text>
        </g>
      {/each}
    </svg>
  </div>
</div>
