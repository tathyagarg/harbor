<script lang="ts">
  import Heading from "$lib/components/Heading.svelte";
  import { animate } from "animejs";
  import { onMount } from "svelte";

  const colorMap = [
    "var(--color-emphasis-2)",
    "var(--color-emphasis-3)",
    "var(--color-emphasis-1)",
    "var(--color-emphasis-1)",
    "var(--color-emphasis-3)",
    "var(--color-emphasis-1)",
    "var(--color-emphasis-1)",
  ];

  onMount(async () => {
    for (const comp of htmlComps) {
      comp.classList.add("rounded");
      comp.classList.add("px-2");
    }

    let i = 0;
    for (const comp of htmlComps) {
      await animate(comp, {
        scale: [1, 1.25, 1],
        duration: 750,
        ease: "inOut",
        onBegin: async () => {
          let color = colorMap[i++];

          comp.style.backgroundColor = `rgba(from ${color} r g b / 0.25)`;
          comp.style.border = `1px solid ${color}`;

          await animate(layoutComps[i - 1], {
            opacity: 0,
            translateX: -100,
            duration: 0,
          });

          htmlHiddens[i - 1] = false;

          await animate(layoutComps[i - 1], {
            opacity: 1,
            translateX: 0,
            duration: 250,
          });

          console.log(htmlHiddens);
        },
      }).then(() => {
        comp.style.backgroundColor = "";
        comp.style.border = "";
      });
    }
  });

  let htmlHiddens = $state([
    true, // body
    true, // div
    true, // h1
    true, // p
    true, // ol
    true, // li
    true, // li
  ]);

  let htmlComps: HTMLDivElement[] = $state([]);
  let layoutComps: HTMLDivElement[] = $state([]);
</script>

<div class="w-full h-full grid grid-cols-3">
  <div class="w-full h-full flex flex-col p-1">
    <span class="mb-2 pb-1 border-b-1 border-emphasis-1/25">
      <Heading text="DOM Tree" />
    </span>
    <div
      class="flex-1 rounded border-1 border-emphasis-1/25 font-mono p-2 text-sm *:my-2
      bg-bg"
    >
      <div class="text-emphasis-2" bind:this={htmlComps[0]}>
        <span>body</span>
      </div>
      <div class="ml-6" bind:this={htmlComps[1]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-3">div</span>
      </div>
      <div class="ml-12" bind:this={htmlComps[2]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-1">h1</span>
      </div>
      <div class="ml-12" bind:this={htmlComps[3]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-1">p</span>
      </div>
      <div class="ml-6" bind:this={htmlComps[4]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-3">ol</span>
      </div>
      <div class="ml-12" bind:this={htmlComps[5]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-1">li</span>
      </div>
      <div class="ml-12" bind:this={htmlComps[6]}>
        <span class="text-emphasis-1/25"> &mdash; </span>
        <span class="text-emphasis-1">li</span>
      </div>
    </div>
  </div>
  <div class="w-full h-full col-span-2 p-1 flex flex-col">
    <span class="mb-2 pb-1 border-b-1 border-emphasis-1/25">
      <Heading text="Layout" />
    </span>
    <div
      class="flex-1 rounded border-1 border-emphasis-1/25 overflow-hidden font-mono lowercase
      bg-bg
      "
    >
      <div
        class="w-full h-full bg-emphasis-2/5 border-2 border-emphasis-2 rounded
        p-1 text-emphasis-2"
        class:hidden={htmlHiddens[0]}
        bind:this={layoutComps[0]}
      >
        Body

        <div
          class="w-full bg-emphasis-3/5 border-2 border-emphasis-3 rounded
          p-1 text-emphasis-3 mt-2"
          class:hidden={htmlHiddens[1]}
          bind:this={layoutComps[1]}
        >
          div

          <div
            class="w-full bg-emphasis-1/5 border-2 border-emphasis-1 rounded
            p-1 text-emphasis-1 mt-2"
            class:hidden={htmlHiddens[2]}
            bind:this={layoutComps[2]}
          >
            h1
          </div>

          <div
            class="w-full bg-emphasis-1/5 border-2 border-emphasis-1 rounded
            p-1 text-emphasis-1 mt-2"
            class:hidden={htmlHiddens[3]}
            bind:this={layoutComps[3]}
          >
            p
          </div>
        </div>

        <div
          class="w-full bg-emphasis-3/5 border-2 border-emphasis-3 rounded
            p-1 text-emphasis-3 mt-2"
          class:hidden={htmlHiddens[4]}
          bind:this={layoutComps[4]}
        >
          ol

          <div
            class="w-full bg-emphasis-1/5 border-2 border-emphasis-1 rounded
              p-1 text-emphasis-1 mt-2"
            class:hidden={htmlHiddens[5]}
            bind:this={layoutComps[5]}
          >
            li
          </div>

          <div
            class="w-full bg-emphasis-1/5 border-2 border-emphasis-1 rounded
              p-1 text-emphasis-1 mt-2"
            class:hidden={htmlHiddens[6]}
            bind:this={layoutComps[6]}
          >
            li
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
