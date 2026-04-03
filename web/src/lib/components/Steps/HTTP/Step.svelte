<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";

  let serverColor = $state("#f00");

  onMount(() => {
    setInterval(() => {
      serverColor = serverColor === "#f00" ? "#0f0" : "#f00";
    }, 1000);
  });
</script>

<div class="h-full w-full flex items-center justify-center px-24 text-subtext">
  <svg viewBox="0 0 400 200">
    <defs>
      <filter id="client-glow" x="-50%" y="-50%" width="200%" height="200%">
        <feDropShadow
          dx="0"
          dy="0"
          stdDeviation="2"
          flood-color="rgba(from var(--color-pretty-blue) r g b / 75%)"
          flood-opacity="1"
        />
      </filter>

      <filter id="server-glow" x="-50%" y="-50%" width="200%" height="200%">
        <feDropShadow
          dx="0"
          dy="0"
          stdDeviation="2"
          flood-color="rgba(from var(--color-gh-green) r g b / 75%)"
          flood-opacity="1"
        />
      </filter>
    </defs>

    <g>
      <rect
        x="10"
        y="75"
        width="100"
        height="50"
        fill="rgba(from var(--color-pretty-blue) r g b / 15%)"
        stroke="var(--color-pretty-blue)"
        stroke-width="2"
        filter="url(#client-glow)"
        rx="8"
        ry="8"
      />
      <text
        x="60"
        y="100"
        font-size="14"
        class="text-center"
        dominant-baseline="middle"
        text-anchor="middle"
        fill="var(--color-pretty-blue)"
        font-family="var(--font-code)"
      >
        [ client ]
      </text>
    </g>

    <!-- arrow -->
    <path
      d="M 100 50 C 150 0, 250 0, 300 50"
      stroke="#eee"
      stroke-width="2"
      fill="none"
    />
    <polygon
      points="295,45 305,50 295,55"
      transform="translate(90 -165) rotate(36 0 0)"
      fill="#eee"
    />

    <polygon
      points="95,145 105,150 95,155"
      transform="translate(55 322) rotate(227 0 0)"
      fill="#eee"
    />
    <path
      d="M 100 150 C 150 200, 250 200, 300 150"
      stroke="#eee"
      stroke-width="2"
      fill="none"
    />

    <g>
      <rect
        x="290"
        y="75"
        width="100"
        height="50"
        fill="rgba(from var(--color-gh-green) r g b / 15%)"
        stroke="var(--color-gh-green)"
        stroke-width="2"
        filter="url(#server-glow)"
        rx="8"
        ry="8"
      />

      <text
        x="340"
        y="100"
        font-size="14"
        class="text-center"
        dominant-baseline="middle"
        text-anchor="middle"
        fill="var(--color-gh-green)"
        font-family="var(--font-code)"
      >
        [ server ]
      </text>
    </g>
  </svg>
</div>
