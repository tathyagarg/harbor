<script>
  import Step from "$lib/components/Step.svelte";
  import TTFLoader from "$lib/components/steps/TTFLoader.svelte";

  const steps = [
    "TTF Loader",
    "HTTP Client",
    "HTML Parser -> DOM",
    "Link Resolution",
    "CSS Parser -> CSSOM",
    "Style Resolution & Cascading",
    "Layout Engine (Box Model)",
    "TTF Rasterizer",
    "Painting",
  ];

  let selected_step = $state(0);
</script>

<div class="py-4 flex flex-row gap-8">
  <ol class="list-decimal pl-4 flex-1">
    {#each steps as step, i}
      <Step
        title={step}
        onClick={() => {
          console.log("clicked", i);
          selected_step = i;
        }}
        last={i === steps.length - 1}
      />
    {/each}
  </ol>

  <div class="flex-2 bg-(--celadon)/25 rounded-lg">
    {#if selected_step === 0}
      <TTFLoader />
    {/if}
  </div>
</div>
