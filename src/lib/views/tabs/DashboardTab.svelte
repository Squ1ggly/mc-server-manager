<script lang="ts">
  import { api, type ServerAddress, type ServerConfig } from "../../api";
  import { serversStore } from "../../stores/servers.svelte";
  import { statsStore } from "../../stores/stats.svelte";
  import { toastsStore } from "../../stores/toasts.svelte";
  import { formatBytes, formatDateTime, formatUptime } from "../../format";
  import { STATUS_META } from "../../status";
  import StatTile from "../../components/StatTile.svelte";
  import Sparkline from "../../components/Sparkline.svelte";

  interface Props {
    server: ServerConfig;
  }

  let { server }: Props = $props();

  const BYTES_PER_MB = 1024 * 1024;

  const status = $derived(serversStore.statusOf(server.id));
  const statusMeta = $derived(STATUS_META[status]);
  const players = $derived(serversStore.playersOf(server.id));
  const stats = $derived(statsStore.of(server.id));
  const isLive = $derived(stats.latest !== null);
  const isProxy = $derived(server.loader === "velocity" || server.loader === "bungeecord");

  let address = $state<ServerAddress | null>(null);

  $effect(() => {
    api
      .getServerAddress(server.id)
      .then((result) => (address = result))
      .catch(() => (address = null));
  });

  const lanAddress = $derived(address ? `${address.lanIp}:${address.port}` : "—");

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toastsStore.success("Copied to clipboard 📋");
    } catch (error) {
      toastsStore.error(String(error));
    }
  }

  // Fixed memory scale relative to the configured heap keeps the sparkline
  // stable; the JVM can exceed -Xmx with off-heap memory, hence the headroom.
  const memoryScaleMax = $derived(server.memoryMb * BYTES_PER_MB * 1.5);

  const cpuText = $derived(isLive ? `${stats.latest!.cpuPercent.toFixed(1)} %` : "—");
  const memoryText = $derived(isLive ? formatBytes(stats.latest!.memoryBytes) : "—");
  const uptimeText = $derived(isLive ? formatUptime(stats.latest!.uptimeSeconds) : "—");
  const createdText = $derived(formatDateTime(server.createdAtUnix));
</script>

<div class="dash">
  {#if !isProxy}
    <button class="address" onclick={() => copy(lanAddress)} title="Click to copy">
      <span class="address-label">🔗 LAN address — click to copy</span>
      <span class="address-value">{lanAddress}</span>
    </button>
  {/if}

  <div class="grid">
    <StatTile label="Status" value="{statusMeta.label} {statusMeta.emoji}" />
    <StatTile
      label="Players online"
      value={String(players.length)}
      sub={players.length > 0 ? players.slice(0, 5).join(", ") : "nobody here yet 🌙"}
    />
    <StatTile label="Uptime" value={uptimeText} />
    <StatTile label="Version" value={server.mcVersion} sub={server.loader} />
    <StatTile
      label="CPU"
      value={cpuText}
      sub={isLive ? "of the whole machine" : "start the server to see stats"}
    >
      {#if stats.cpuHistory.length > 1}
        <Sparkline values={stats.cpuHistory} max={100} color="var(--chart-cpu)" />
      {/if}
    </StatTile>
    <StatTile label="Memory" value={memoryText} sub="allocated: {server.memoryMb} MB">
      {#if stats.memoryHistory.length > 1}
        <Sparkline values={stats.memoryHistory} max={memoryScaleMax} color="var(--chart-mem)" />
      {/if}
    </StatTile>
    <StatTile label="Created" value={createdText} />
  </div>
</div>

<style>
  .dash {
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
    padding: 0.25rem 0.25rem 1rem;
  }

  .address {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: flex-start;
    text-align: left;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-soft);
    padding: 0.9rem 1.1rem;
    cursor: pointer;
    font-family: inherit;
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .address:hover {
    border-color: color-mix(in srgb, var(--border) 40%, var(--accent));
  }

  .address-label {
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
  }

  .address-value {
    font-family: var(--font-pixel);
    font-size: 1.35rem;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
    gap: 1.2rem;
  }
</style>
