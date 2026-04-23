<script lang="ts">
  let { label, value } = $props();
</script>

<div class="font-mono text-[12px]">
  <span class="text-emphasis-1/75">
    {label}:
  </span>
  <span class="text-current">
    {value}
  </span>
  <span class="text-emphasis-1/75">;</span>
</div>
