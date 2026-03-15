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
      tertiaryColor: "#13293d",
    },
    flowchart: {
      useMaxWidth: true,
    },
  });

  onMount(() => {
    mermaid.contentLoaded();
  });
</script>

<div
  class="flex flex-row gap-8 items-center justify-center w-full h-full p-8 mx-auto"
>
  <div class="flex-3 flex flex-col items-center">
    <h1>DOM Structure</h1>
    <div class="w-full rounded-lg p-4">
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

  <span class="flex-1 text-6xl text-(--text) text-center">→</span>

  <div class="flex flex-col items-center">
    <h1>Box Layout Structure</h1>
    <div class="w-full rounded-lg p-4 flex items-center justify-center">
      <pre class="mermaid">
flowchart TD
  subgraph "Body"
    direction TD
  A["`Heading (h1)
pos(x1, y1)
dims(w1, h1)`"] ~~~ B["`Paragraph (p)
pos(x2, y2)
dims(w2, h2)`"]
  end

    </pre>
    </div>
  </div>
</div>
