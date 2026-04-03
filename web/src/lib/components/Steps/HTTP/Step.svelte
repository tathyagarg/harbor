<script lang="ts">
  import { animate, createTimeline, onScroll } from "animejs";
  import { onMount } from "svelte";

  let packetColor = $state("rgba(from var(--color-subtext) r g b / 25%)");
  let animationState = $state("client");

  onMount(async () => {
    animate("#packet", {
      cx: 120,
      duration: 0,
    });

    let timeline = createTimeline({
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
    });

    packetColor = "rgba(from var(--color-pretty-blue) r g b / 75%)";

    timeline
      .add("#packet", {
        cx: 280,
        duration: 1000,
        easing: "outCirc",
        loop: 4,
        delay: 1000,
      })
      .call(() => {
        packetColor = "rgba(from var(--color-gh-green) r g b / 75%)";
        animationState = "server";
      })
      .add("#packet", {
        cx: 120,
        duration: 1000,
        easing: "outCirc",
        loop: 4,
        delay: 500,
      });
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

    <circle
      cx="120"
      cy="100"
      r="5"
      fill={packetColor}
      filter={`url(#${animationState}-glow)`}
      id="packet"
    />

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
