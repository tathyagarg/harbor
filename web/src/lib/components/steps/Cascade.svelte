<script>
  import { primaryColor, primaryTextColor } from "$lib";
  import { Mermaid } from "@friendofsvelte/mermaid";
</script>

<div class="relative w-full top-[7.5%] bg-(--cerulean)/75 rounded-lg">
  <div class="mx-auto max-w-[60%]">
    <Mermaid
      string={`
flowchart TD
  subgraph "body"
    H[font-size: 16px]
  end

  subgraph "div"
    A[position]
    B[color]
    C[font-size]
    G[width: 5em]
  end

  subgraph "span"
    D[position]
    E[color]
    F[font-size]
  end

  B -.->|Cascade| E
  C -.->|Cascade| F
  H -.->|Cascade| C
  H ---|Used| G
  G ==>|Resolve| I[width: 80px]

  classDef div color:black
  classDef span color:black
  classDef misc color:black

  class A,B,C,G div
  class D,E,F span
  class H,I misc
`}
      config={{
        theme: "base",
        themeVariables: {
          primaryColor,
          primaryTextColor,
          secondaryColor: "#13293d",
          nodeTextColor: "#fff",
          tertiaryColor: "#8fb99680",
          titleColor: primaryTextColor,
        },
      }}
    />
  </div>
</div>
