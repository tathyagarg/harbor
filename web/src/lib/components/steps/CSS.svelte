<script lang="ts">
  import { Mermaid } from "@friendofsvelte/mermaid";
</script>

<div class="flex flex-row gap-8 items-center justify-center w-full h-full">
  <div>
    <h1>CSS Data</h1>
    <div
      class="text-8xl font-bold text-center mb-4 bg-(--text) text-black px-4 aspect-[4/5] flex items-center justify-center rounded-lg"
    >
      <div>&lcub; &rcub;</div>
    </div>
  </div>

  <span class="text-6xl text-(--text)">→</span>

  <div>
    <h1>CSSOM Structure</h1>
    <Mermaid
      string={`
    flowchart TD
      A[CSSStyleSheet] --> B[StyleSheet]
      B --> C{CSSRules}
      C --> D[CSSStyleRule]
      D --> E[SelectorList]
      D --> F[DeclarationList]
      C --> G[CSSImportRule]
      C --> H[CSSMediaRule]
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
