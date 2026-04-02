<script lang="ts">
  import Icon from "@iconify/svelte";
  import { animate, createTimeline, stagger } from "animejs";
  import { onMount } from "svelte";
  import Table from "./Table.svelte";
  import Line from "./Line.svelte";

  let timeline = createTimeline();

  let line: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    color: string;
  } | null = $state(null);

  onMount(() => {
    animate(".line", {
      width: "0",
      duration: 0,
    });

    animate("#tables", {
      opacity: 0,
      translateX: 300,
      duration: 0,
    });

    timeline
      .label("start")
      .label("fadeaway", "+=1000")
      .label("lines", "+=2000")
      .label("tables", "+=4000")
      .add(
        "#sub-fontfile",
        {
          opacity: [0, 1],
          scale: [0.25, 1],
          duration: 500,
          easing: "easeInOutQuad",
        },
        "start",
      )
      .add(
        "#fontfile",
        {
          background: "#333",
          duration: 500,
          easing: "easeInOutQuad",
        },
        "fadeaway",
      )
      .add(
        "#filetext",
        {
          opacity: [1, 0],
          duration: 200,
          easing: "easeInOutQuad",
        },
        "fadeaway",
      )
      .add(
        ".line",
        {
          width: ["0", "90%"],
          duration: 500,
          easing: "easeInOutQuad",
          delay: stagger(100),
        },
        "lines",
      )
      .add(
        "#sub-fontfile",
        {
          translateX: [0, -150],
          duration: 500,
          easing: "easeInOutQuad",
        },
        "tables",
      )
      .add(
        "#tables",
        {
          translateX: [300, 150],
          opacity: [0, 1],
          duration: 500,
          easing: "easeInOutQuad",
        },
        "tables",
      );
  });
</script>

<div class="w-full h-full relative" id="step">
  <div
    class="w-fit flex flex-col items-center justify-center absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
    id="sub-fontfile"
  >
    <div
      class="bg-text text-text-dark text-center w-[10vw] rounded-md
    aspect-[4/5] flex items-center justify-center relative overflow-hidden
    cursor-default
    "
      id="fontfile"
    >
      <span class="text-8xl" id="filetext"> Aa </span>
      <div class="absolute top-0 left-0 w-full h-full">
        <div
          id="red-lines"
          class="relative top-2 h-3"
          onmouseover={() => {
            const redLines = document.querySelector("#red-lines");
            const headTable = document.querySelector("#head");
            const stepContainer = document.querySelector("#step-container");

            if (redLines && headTable && stepContainer) {
              const redRect = redLines.getBoundingClientRect();
              const headRect = headTable.getBoundingClientRect();
              const fontRect = stepContainer.getBoundingClientRect();

              console.log(redRect, headRect, fontRect);

              line = {
                x1: redRect.right - fontRect.left - 10,
                y1: redRect.top + redRect.height / 2 - fontRect.top,
                x2: headRect.left - fontRect.left,
                y2: headRect.top + headRect.height / 2 - fontRect.top,
                color: "var(--color-scratch-red)",
              };

              console.log("mouseover");
            }
          }}
          onmouseleave={() => (line = null)}
          onfocus={() => console.log("focus")}
          role="button"
          tabindex="0"
        >
          <Line color="var(--color-scratch-red)" index={0} />
        </div>
        <div
          id="blue-lines"
          class="relative top-3 h-7"
          onmouseover={() => {
            const blueLines = document.querySelector("#blue-lines");
            const cmapTable = document.querySelector("#cmap");
            const stepContainer = document.querySelector("#step-container");

            if (blueLines && cmapTable && stepContainer) {
              const blueRect = blueLines.getBoundingClientRect();
              const cmapRect = cmapTable.getBoundingClientRect();
              const fontRect = stepContainer.getBoundingClientRect();

              console.log(blueRect, cmapRect, fontRect);

              line = {
                x1: blueRect.right - fontRect.left - 10,
                y1: blueRect.top + blueRect.height / 2 - fontRect.top,
                x2: cmapRect.left - fontRect.left,
                y2: cmapRect.top + cmapRect.height / 2 - fontRect.top,
                color: "var(--color-pretty-blue)",
              };

              console.log("mouseover");
            }
          }}
          onmouseleave={() => (line = null)}
          onfocus={() => console.log("focus")}
          role="button"
          tabindex="0"
        >
          {#each Array(2) as _, i}
            <Line color="var(--color-pretty-blue)" index={i} />
          {/each}
        </div>
        <div
          id="green-lines"
          class="relative top-4 h-19"
          onmouseover={() => {
            const greenLines = document.querySelector("#green-lines");
            const glyfTable = document.querySelector("#glyf");
            const stepContainer = document.querySelector("#step-container");

            if (greenLines && glyfTable && stepContainer) {
              const greenRect = greenLines.getBoundingClientRect();
              const glyfRect = glyfTable.getBoundingClientRect();
              const fontRect = stepContainer.getBoundingClientRect();

              console.log(greenRect, glyfRect, fontRect);

              line = {
                x1: greenRect.right - fontRect.left - 10,
                y1: greenRect.top + greenRect.height / 2 - fontRect.top,
                x2: glyfRect.left - fontRect.left,
                y2: glyfRect.top + glyfRect.height / 2 - fontRect.top,
                color: "var(--color-gh-green)",
              };

              console.log("mouseover");
            }
          }}
          onmouseleave={() => (line = null)}
          onfocus={() => console.log("focus")}
          role="button"
          tabindex="0"
        >
          {#each Array(5) as _, i}
            <Line color="var(--color-gh-green)" index={i} />
          {/each}
        </div>
        <div
          id="grey-lines"
          class="relative top-5 h-3"
          onmouseover={() => {
            const greyLines = document.querySelector("#grey-lines");
            const otherTable = document.querySelector("#other");
            const stepContainer = document.querySelector("#step-container");

            if (greyLines && otherTable && stepContainer) {
              const greyRect = greyLines.getBoundingClientRect();
              const otherRect = otherTable.getBoundingClientRect();
              const fontRect = stepContainer.getBoundingClientRect();

              console.log(greyRect, otherRect, fontRect);

              line = {
                x1: greyRect.right - fontRect.left - 10,
                y1: greyRect.top + greyRect.height / 2 - fontRect.top,
                x2: otherRect.left - fontRect.left,
                y2: otherRect.top + otherRect.height / 2 - fontRect.top,
                color: "var(--color-subtext)",
              };

              console.log("mouseover");
            }
          }}
          onmouseleave={() => (line = null)}
          onfocus={() => console.log("focus")}
          role="button"
          tabindex="0"
        >
          <Line color="var(--color-subtext)" index={0} />
        </div>
      </div>
    </div>
    <p class="m-0">Font.ttf</p>
  </div>

  <div
    class="absolute grid grid-rows-4 gap-4 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
    id="tables"
  >
    <Table name="head" color="var(--color-scratch-red)" />
    <Table name="cmap" color="var(--color-pretty-blue)" />
    <Table name="glyf" color="var(--color-gh-green)" />
    <Table name="other" color="var(--color-subtext)" />
  </div>

  {#if line}
    <svg
      class="absolute left-0 top-0 pointer-events-none"
      style="width: 100%; height: 100%;"
    >
      <line
        x1={line.x1}
        y1={line.y1}
        x2={(line.x1 + line.x2) / 2}
        y2={line.y1}
        stroke={line.color}
        stroke-width="2"
      />
      <line
        x1={(line.x1 + line.x2) / 2}
        y1={line.y1}
        x2={(line.x1 + line.x2) / 2}
        y2={line.y2}
        stroke={line.color}
        stroke-width="2"
      />
      <line
        x1={(line.x1 + line.x2) / 2}
        y1={line.y2}
        x2={line.x2}
        y2={line.y2}
        stroke={line.color}
        stroke-width="2"
      />
    </svg>
  {/if}
</div>
