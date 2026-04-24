<script lang="ts">
  import { STEP_NAMES } from "$lib";
  import Font from "$lib/components/Steps/01_Font/Step.svelte";
  import HTTP from "$lib/components/Steps/02_HTTP/Step.svelte";
  import HTML from "$lib/components/Steps/03_HTML/Step.svelte";
  import Link from "$lib/components/Steps/04_Link/Step.svelte";
  import CSS from "$lib/components/Steps/05_CSS/Step.svelte";
  import Cascade from "$lib/components/Steps/06_Cascade/Step.svelte";
  import Layout from "$lib/components/Steps/07_Layout/Step.svelte";
  import Rasterize from "$lib/components/Steps/08_Rasterize/Step.svelte";
  import Paint from "$lib/components/Steps/09_Paint/Step.svelte";
  import Heading from "./Heading.svelte";

  const steps = [
    Font,
    HTTP,
    HTML,
    Link,
    CSS,
    Cascade,
    Layout,
    Rasterize,
    Paint,
  ];

  let { step } = $props();
  const Component = steps[step];
</script>

<div class="w-full border-1 rounded border-emphasis-1/25 my-4 overflow-hidden">
  <div class="w-full bg-emphasis-1/10 p-2">
    <Heading text={`Step ${step + 1}: ${STEP_NAMES[step]}`} />
  </div>
  <div class="w-full bg-bg border-t-1 border-emphasis-1/25 p-2">
    <Component />
  </div>
</div>
