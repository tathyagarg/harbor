<script lang="ts">
  import Heading from "$lib/components/Heading.svelte";
  import { onMount } from "svelte";
  import Declaration from "./Declaration.svelte";
  import { animate, stagger } from "animejs";

  onMount(async () => {
    let declarations = [uaDecl, authorDecl, importantDecl];
    animate(declarations, {
      opacity: 0,
      translateX: -20,
      duration: 0,
    });

    await animate(".order-card", {
      opacity: 0,
      translateY: 20,
      duration: 0,
    });

    await new Promise((resolve) => setTimeout(resolve, 500));

    await animate(".order-card", {
      translateY: [20, 0],
      opacity: [0, 1],
      delay: stagger(100),
      ease: "outQuad",
    });

    for (const card of [uaCard, authorCard, importantCard]) {
      await animate(card, {
        scale: [1, 1.05, 1.05, 1],
        boxShadow: [
          "none",
          "0 0 8px 1px currentColor",
          "0 0 8px 1px currentColor",
          "none",
        ],
        delay: 500,
        duration: 1000,
        onBegin: () => {
          const decl = declarations.shift();
          if (decl) {
            animate(decl, {
              opacity: [0, 1],
              translateX: [-20, 0],
              duration: 500,
              ease: "outQuad",
            });
          }
        },
      });
    }
  });

  let uaCard: HTMLDivElement;
  let authorCard: HTMLDivElement;
  let importantCard: HTMLDivElement;

  let uaDecl: HTMLSpanElement;
  let authorDecl: HTMLSpanElement;
  let importantDecl: HTMLSpanElement;
</script>

<div class="w-full h-full flex gap-2 p-2">
  <div class="w-full h-full flex flex-col gap-2">
    <span class="text-[12px] border-b-1 border-emphasis-1/25 pb-1 mb-1">
      <Heading text="Cascade Order" />
    </span>
    <div
      class="flex-1 p-2 border-emphasis-1/25 border-1 bg-bg rounded order-card z-10"
      bind:this={uaCard}
    >
      <div
        class="flex justify-between border-b-1 border-emphasis-1/25 pb-2 mb-2"
      >
        <Heading text="User-Agent" upper={false} fontSize="16px" />
        <Heading text="Lowest" />
      </div>
      <Declaration label="font-weight" value="bold" />
    </div>

    <div
      class="flex-1 p-2 border-emphasis-1/25 border-1 bg-bg rounded text-pretty-blue order-card z-10"
      bind:this={authorCard}
    >
      <div
        class="flex justify-between border-b-1 border-emphasis-1/25 pb-2 mb-2 text-pretty-blue"
      >
        <Heading
          text="Author"
          upper={false}
          fontSize="16px"
          defaultColor={false}
        />
        <Heading text="Medium" />
      </div>
      <Declaration label="color" value="blue" />
    </div>

    <div
      class="flex-1 p-2 border-emphasis-1/25 border-1 bg-bg rounded text-scratch-red order-card z-10"
      bind:this={importantCard}
    >
      <div
        class="flex justify-between border-b-1 border-emphasis-1/25 pb-2 mb-2"
      >
        <Heading
          text="!important"
          upper={false}
          fontSize="16px"
          defaultColor={false}
        />
        <Heading text="Highest" />
      </div>
      <Declaration label="font-size" value="2em !important" />
    </div>
  </div>

  <div class="w-full h-full flex flex-col gap-2">
    <span class="text-[12px] border-b-1 border-emphasis-1/25 pb-1 mb-1">
      <Heading text="Computed Style" />
    </span>
    <div
      class="flex-1 flex flex-col rounded border-1 border-emphasis-1/25 font-mono overflow-hidden"
    >
      <span
        class="text-emphasis-1/75 bg-bg p-2 border-b-1 border-emphasis-1/25"
      >
        element &lcub;
      </span>

      <div class="flex-1 p-2 flex flex-col gap-1 justify-center">
        <span bind:this={uaDecl}>
          <Declaration label="font-weight" value="700" />
        </span>
        <span bind:this={authorDecl}>
          <Declaration label="color" value="#00f" />
        </span>
        <span bind:this={importantDecl}>
          <Declaration label="font-size" value="32px" />
        </span>
      </div>

      <span
        class="text-emphasis-1/75 bg-bg p-2 border-t-1 border-emphasis-1/25"
      >
        &rcub;
      </span>
    </div>
  </div>
</div>
