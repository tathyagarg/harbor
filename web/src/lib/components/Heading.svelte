<script lang="ts">
  let { text, fontSize = "12px", upper = true, defaultColor = true } = $props();
</script>

<span
  class="font-mono font-bold"
  class:text-emphasis-1={defaultColor}
  class:uppercase={upper}
  style="font-size: {fontSize}"
>
  {text}
</span>
