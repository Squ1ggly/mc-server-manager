<script lang="ts">
  // The ServerForge mark — a little pixel server tower, drawn by us.
  // Kept visually in sync with assets/app-icon.svg, which is the source
  // the Store/desktop icon sizes are generated from.

  interface Props {
    size?: number;
  }

  let { size = 22 }: Props = $props();

  // 16x16 pixel grid: [color, x, y, width, height].
  const PIXEL_RECTS: [string, number, number, number, number][] = [
    // Tower chassis
    ["#454c5c", 3, 2, 10, 12],
    ["#59627a", 3, 2, 10, 1],
    ["#525a6d", 3, 2, 1, 12],
    ["#3a4050", 12, 2, 1, 12],
    // Drive bays: dark slot, green status LED, vent grille
    ["#2b303b", 4, 3, 8, 2],
    ["#7ed957", 5, 3, 1, 1],
    ["#59627a", 7, 3, 4, 1],
    ["#2b303b", 4, 6, 8, 2],
    ["#7ed957", 5, 6, 1, 1],
    ["#59627a", 7, 6, 4, 1],
    ["#2b303b", 4, 9, 8, 2],
    ["#7ed957", 5, 9, 1, 1],
    ["#59627a", 7, 9, 4, 1],
    // Power bar along the base
    ["#5cbf3f", 4, 12, 8, 1],
  ];
</script>

<svg
  viewBox="0 0 16 16"
  width={size}
  height={size}
  style:image-rendering="pixelated"
  shape-rendering="crispEdges"
  aria-hidden="true"
>
  <rect x="0" y="0" width="16" height="16" rx="2.5" fill="#23262e" />
  {#each PIXEL_RECTS as [color, x, y, width, height], index (index)}
    <rect {x} {y} {width} {height} fill={color} />
  {/each}
</svg>

<style>
  svg {
    display: block;
  }
</style>
