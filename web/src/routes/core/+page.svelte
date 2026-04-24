<script lang="ts">
  import Footer from "$lib/components/Footer.svelte";
  import StepContainer from "$lib/components/StepContainer.svelte";
</script>

<div class="w-1/2 min-h-screen mx-auto py-8">
  <h1 class="text-emphasis-2 text-6xl my-4">HTML</h1>
  <p class="text-subtext">The HTML core component</p>

  <StepContainer step={2} />
  <StepContainer step={6} />
</div>

<Footer />
