<script lang="ts">
  import { animate, createTimeline, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";

  onMount(async () => {
    let timeline = createTimeline({
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
      defaults: {
        duration: 500,
        ease: "easeInOutQuad",
      },
    });

    animate(["#html", "#body", "#h1", "#p1", "#a", "#p3"], {
      width: 0,
      height: 0,
      duration: 0,
    });

    animate(
      [
        "#html-text",
        "#body-text",
        "#h1-text",
        "#p1-text",
        "#a-text",
        "#p3-text",
      ],
      {
        opacity: 0,
        duration: 0,
      },
    );

    timeline
      .add("#filename", {
        opacity: [1, 0],
        translateY: [0, 20],
        delay: 1000,
      })
      .add(
        "#file-container",
        {
          x: [140, 20],
          width: [120, 360],
        },
        1500,
      )
      .add(
        "#html",
        {
          width: 360,
          height: 150,
        },
        2000,
      )
      .add(
        "#html-text",
        {
          opacity: 1,
        },
        2750,
      )
      .add(
        "#body",
        {
          width: 350,
          height: 115,
        },
        2750,
      )
      .add(
        "#body-text",
        {
          opacity: 1,
        },
        3000,
      )
      .add(
        "#h1",
        {
          width: 340,
          height: 30,
        },
        3250,
      )
      .add(
        "#h1-text",
        {
          opacity: 1,
        },
        3250,
      )
      .add(
        ["#p1", "#a", "#p3"],
        {
          width: 110,
          height: 40,
          delay: stagger(250),
        },
        3750,
      )
      .add(
        ["#p1-text", "#a-text", "#p3-text"],
        {
          opacity: 1,
          delay: stagger(250),
        },
        3750,
      );
  });
</script>

<div class="h-full w-full flex items-center justify-center px-24 text-subtext">
  <svg class="" viewBox="0 0 400 200">
    <rect
      x="140"
      y="25"
      width="120"
      height="150"
      fill="transparent"
      stroke="rgba(from var(--color-emphasis-1) r g b / 25%)"
      stroke-width="1"
      rx="8"
      ry="8"
      id="file-container"
    />

    <g>
      <rect
        x="20"
        y="25"
        width="360"
        height="150"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-scratch-red)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="html"
      />
      <text
        id="html-text"
        x="60"
        y="40"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-scratch-red)">{"[ <html> ]"}</tspan>
      </text>
    </g>

    <g>
      <rect
        x="25"
        y="55"
        width="350"
        height="115"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-pretty-blue)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="body"
      />
      <text
        id="body-text"
        x="65"
        y="70"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-pretty-blue)">{"[ <body> ]"}</tspan>
      </text>
    </g>

    <g>
      <rect
        x="30"
        y="85"
        width="340"
        height="30"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-gh-green)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="h1"
      />

      <text
        id="h1-text"
        x="65"
        y="100"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-gh-green)">{`[ <h1> ]`}</tspan>
      </text>
    </g>

    <g>
      <rect
        x="30"
        y="120"
        width="110"
        height="40"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-subtext)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="p1"
      />
      <text
        id="p1-text"
        x="85"
        y="140"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-subtext)">{`[ <p> ]`}</tspan>
      </text>

      <rect
        x="145"
        y="120"
        width="110"
        height="40"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-subtext)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="a"
      />
      <text
        id="a-text"
        x="200"
        y="140"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-subtext)">{`[ <a> ]`}</tspan>
      </text>

      <rect
        x="260"
        y="120"
        width="110"
        height="40"
        fill="rgba(from var(--color-text-dark) r g b / 25%)"
        stroke="var(--color-subtext)"
        stroke-width="1"
        rx="8"
        ry="8"
        id="p3"
      />
      <text
        id="p3-text"
        x="315"
        y="140"
        text-anchor="middle"
        fill="currentColor"
        font-size="12"
        font-family="monospace"
        dominant-baseline="middle"
      >
        <tspan fill="var(--color-subtext)">{`[ <p> ]`}</tspan>
      </text>
    </g>

    <text
      id="filename"
      x="200"
      y="190"
      text-anchor="middle"
      fill="currentColor"
      font-size="12"
      font-family="monospace"
      dominant-baseline="middle"
    >
      index.html
    </text>
  </svg>
</div>
