<script lang="ts">
  import { primaryColor, primaryTextColor } from "$lib";
  import mermaid from "mermaid";
  import { onMount } from "svelte";

  mermaid.initialize({
    startOnLoad: true,
    htmlLabels: true,
    theme: "base",
    themeVariables: {
      primaryColor,
      primaryTextColor,
      secondaryColor: "#13293d",
      nodeTextColor: "#000",
      tertiaryColor: "#8fb99680",
      titleColor: primaryTextColor,
    },
  });

  onMount(() => {
    mermaid.contentLoaded();
  });
</script>

<div class="flex flex-row gap-8 items-center justify-center w-full h-full">
  <div>
    <h1>HTML Data</h1>
    <div
      class="text-8xl font-bold text-center mb-4 bg-(--text) text-black px-4 aspect-[4/5] flex items-center justify-center rounded-lg"
    >
      <div>&lt;/&gt;</div>
    </div>
  </div>

  <span class="text-6xl text-(--text)">→</span>

  <div class="w-[50%] rounded-lg p-4">
    <h1>DOM Structure</h1>
    <pre class="mermaid">
    graph TD
      A[Document] --> B[HTML]
      B --> C&lcub;Head&rcub;
      B --> D&lcub;Body&rcub;
      C --> E[Title]
      C --> F[Meta]
      D --> G[Headings]
      D --> H[Paragraphs]
      D --> I[etc.]
    </pre>
  </div>
</div>
