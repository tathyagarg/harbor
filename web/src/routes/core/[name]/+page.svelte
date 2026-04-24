<script lang="ts">
  import { STEP_NAMES } from "$lib";
  import Footer from "$lib/components/Footer.svelte";
  import StepContainer from "$lib/components/StepContainer.svelte";

  let { data } = $props();
  let { title, description, steps } = $derived(data);
</script>

<div class="w-1/2 min-h-screen mx-auto py-8">
  <h1 class="text-emphasis-2 text-6xl my-4">{title}</h1>
  <p class="text-subtext">{description}</p>

  <hr class="text-emphasis-1/25 my-2" />

  {#each steps as step}
    <h1 class="text-emphasis-2 text-2xl mb-2">
      Step {step.index + 1}: {STEP_NAMES[step.index]}
    </h1>

    <p>{step.description}</p>
    <StepContainer step={step.index} />
  {/each}
</div>

<Footer />
