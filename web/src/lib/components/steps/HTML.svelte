<script lang="ts">
  import { Mermaid } from "@friendofsvelte/mermaid";
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

  <div>
    <h1>DOM Structure</h1>
    <Mermaid
      string={`
    graph TD
      A[Document] --> B[HTML]
      B --> C{Head}
      B --> D{Body}
      C --> E[Title]
      C --> F[Meta]
      D --> G[Headings]
      D --> H[Paragraphs]
      D --> I[etc.]
  `}
      config={{
        theme: "base",
        themeVariables: {
          primaryColor: "#3b82f6",
          primaryTextColor: "#E6F1F7",
        },
      }}
    />
  </div>
</div>
