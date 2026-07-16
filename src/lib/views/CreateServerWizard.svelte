<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { open as openFolderDialog } from "@tauri-apps/plugin-dialog";
  import { api, PROXY_LOADERS, type Loader, type McVersion } from "../api";
  import {
    MEMORY_MAX_MB,
    MEMORY_MIN_MB,
    MEMORY_STEP_MB,
    SERVER_NAME_MAX_LENGTH,
  } from "../constants";
  import { onInstallProgress } from "../events";
  import { serversStore } from "../stores/servers.svelte";
  import { toastsStore } from "../stores/toasts.svelte";
  import Modal from "../components/Modal.svelte";
  import Button from "../components/Button.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";

  interface Props {
    open: boolean;
    onclose: () => void;
  }

  let { open, onclose }: Props = $props();

  interface CatalogEntry {
    value: Loader;
    label: string;
    available: boolean;
  }

  const LOADER_CATALOG: { category: string; entries: CatalogEntry[] }[] = [
    {
      category: "Vanilla & official",
      entries: [
        { value: "vanilla", label: "Vanilla — the official Mojang server", available: true },
        { value: "bds", label: "Bedrock Dedicated Server", available: false },
      ],
    },
    {
      category: "Plugins (Bukkit ecosystem)",
      entries: [
        { value: "paper", label: "Paper — fast, the plugin gold standard", available: true },
        { value: "purpur", label: "Purpur — Paper + extreme configurability", available: true },
        { value: "folia", label: "Folia — multithreaded regions, huge player counts", available: true },
        { value: "spigot", label: "Spigot — the legacy plugin backbone", available: false },
      ],
    },
    {
      category: "Mods",
      entries: [
        { value: "fabric", label: "Fabric — lightweight modern mod loader", available: true },
        { value: "neoforge", label: "NeoForge — modern Forge successor", available: false },
        { value: "forge", label: "Forge — the classic modding giant", available: false },
        { value: "quilt", label: "Quilt — community fork of Fabric", available: false },
      ],
    },
    {
      category: "Hybrid (mods + plugins)",
      entries: [
        { value: "arclight", label: "Arclight — plugins on Forge/Fabric", available: false },
        { value: "mohist", label: "Mohist — Forge modpacks + plugins", available: false },
      ],
    },
    {
      category: "Network proxies",
      entries: [
        { value: "velocity", label: "Velocity — modern secure proxy", available: true },
        { value: "bungeecord", label: "BungeeCord — the original proxy", available: true },
      ],
    },
  ];

  const DEFAULT_GAME_PORT = 25565;
  const DEFAULT_PROXY_PORT = 25577;

  let name = $state("");
  let loader = $state<Loader>("vanilla");
  let versions = $state<McVersion[]>([]);
  let versionsForLoader = $state<Loader | null>(null);
  let selectedVersion = $state("");
  let memoryMb = $state(2048);
  let port = $state(DEFAULT_GAME_PORT);
  let portTouched = $state(false);
  let acceptEula = $state(false);
  let showSnapshots = $state(false);
  let loadingVersions = $state(false);
  let creating = $state(false);
  let progress = $state<number | null>(null);
  let locationParent = $state<string | null>(null);
  let locationPreview = $state("");
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  let advancedOpen = $state(false);
  let javaArgs = $state("");
  let startCommand = $state("");

  const isProxy = $derived(PROXY_LOADERS.includes(loader));
  const hasSnapshots = $derived(versions.some((version) => version.type !== "release"));
  const visibleVersions = $derived(
    versions.filter((version) => showSnapshots || version.type === "release"),
  );
  const canSubmit = $derived(
    name.trim().length > 0 && selectedVersion !== "" && (acceptEula || isProxy) && !creating,
  );

  $effect(() => {
    if (open && versionsForLoader !== loader && !loadingVersions) {
      loadVersions(loader);
    }
  });

  $effect(() => {
    // Sensible port defaults per software family, until the user edits it.
    if (!portTouched) {
      port = isProxy ? DEFAULT_PROXY_PORT : DEFAULT_GAME_PORT;
    }
  });

  $effect(() => {
    // Re-preview the target folder whenever the name or parent changes,
    // debounced so we don't call the backend on every keystroke.
    if (!open) {
      return;
    }
    const currentName = name;
    const currentParent = locationParent;
    clearTimeout(previewTimer);
    previewTimer = setTimeout(async () => {
      try {
        locationPreview = await api.previewServerDir(currentName, currentParent);
      } catch {
        locationPreview = "";
      }
    }, 150);
  });

  onMount(() => {
    const unlistenPromise = onInstallProgress((event) => {
      if (!creating || event.step !== "download-server-jar") {
        return;
      }
      progress =
        event.totalBytes === null ? null : event.downloadedBytes / event.totalBytes;
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  async function loadVersions(forLoader: Loader) {
    loadingVersions = true;
    try {
      versions = await api.listLoaderVersions(forLoader);
      versionsForLoader = forLoader;
      const newestRelease = versions.find((version) => version.type === "release");
      selectedVersion = newestRelease?.id ?? versions[0]?.id ?? "";
    } catch (error) {
      versions = [];
      selectedVersion = "";
      toastsStore.error(`Couldn't load versions: ${error}`);
    } finally {
      loadingVersions = false;
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    creating = true;
    progress = null;
    try {
      const server = await api.createServer({
        name,
        mcVersion: selectedVersion,
        loader,
        memoryMb,
        port,
        acceptEula,
        locationParent,
        javaArgs: javaArgs.trim() === "" ? null : javaArgs.trim(),
        startCommand: startCommand.trim() === "" ? null : startCommand.trim(),
      });
      await serversStore.refresh();
      toastsStore.success(`"${server.name}" is ready! 🎂`);
      resetForm();
      onclose();
    } catch (error) {
      toastsStore.error(String(error));
    } finally {
      creating = false;
    }
  }

  function resetForm() {
    name = "";
    acceptEula = false;
    memoryMb = 2048;
    portTouched = false;
    javaArgs = "";
    startCommand = "";
    advancedOpen = false;
    progress = null;
  }

  function openEula(event: MouseEvent) {
    event.preventDefault();
    openUrl("https://aka.ms/MinecraftEULA").catch(() => {
      toastsStore.show("EULA: https://aka.ms/MinecraftEULA");
    });
  }

  async function browseLocation() {
    const picked = await openFolderDialog({
      directory: true,
      title: "Choose where your servers live",
    });
    if (typeof picked === "string") {
      locationParent = picked;
    }
  }
</script>

<Modal {open} title="New server 🍰" onclose={creating ? undefined : onclose}>
  <form onsubmit={submit}>
    <label>
      <span>Name</span>
      <input
        type="text"
        bind:value={name}
        placeholder="My cozy world"
        maxlength={SERVER_NAME_MAX_LENGTH}
      />
    </label>

    <label>
      <span>Server software</span>
      <select bind:value={loader}>
        {#each LOADER_CATALOG as group (group.category)}
          <optgroup label={group.category}>
            {#each group.entries as entry (entry.value)}
              <option value={entry.value} disabled={!entry.available}>
                {entry.label}{entry.available ? "" : " (coming soon)"}
              </option>
            {/each}
          </optgroup>
        {/each}
      </select>
    </label>

    <div class="row">
      <label class="grow">
        <span>Version</span>
        {#if loadingVersions}
          <div class="loading">Fetching versions… ⛏️</div>
        {:else}
          <select bind:value={selectedVersion}>
            {#each visibleVersions as version (version.id)}
              <option value={version.id}>{version.id}</option>
            {/each}
          </select>
        {/if}
      </label>
      <label class="port-label">
        <span>Port</span>
        <input
          type="number"
          min="1024"
          max="65535"
          bind:value={port}
          oninput={() => (portTouched = true)}
        />
      </label>
    </div>

    {#if hasSnapshots}
      <label class="checkbox">
        <input type="checkbox" bind:checked={showSnapshots} />
        <span>Show snapshots</span>
      </label>
    {/if}

    {#if isProxy}
      <p class="proxy-note">
        Proxies keep their port in their own config (velocity.toml / config.yml) —
        set it there after the first start.
      </p>
    {/if}

    <label>
      <span>Memory — {memoryMb} MB</span>
      <input
        type="range"
        min={MEMORY_MIN_MB}
        max={MEMORY_MAX_MB}
        step={MEMORY_STEP_MB}
        bind:value={memoryMb}
      />
    </label>

    <div class="location">
      <span class="location-label">📁 Save location</span>
      <div class="location-row">
        <span class="location-path" title={locationPreview}>
          {locationPreview || "…"}
        </span>
        <Button variant="soft" onclick={browseLocation}>Browse…</Button>
      </div>
    </div>

    {#if !isProxy}
      <label class="checkbox eula">
        <input type="checkbox" bind:checked={acceptEula} />
        <span>
          I accept the
          <a href="https://aka.ms/MinecraftEULA" onclick={openEula}>Minecraft EULA</a>
        </span>
      </label>
    {/if}

    <div class="advanced">
      <button type="button" class="advanced-toggle" onclick={() => (advancedOpen = !advancedOpen)}>
        <span class="chevron" class:open={advancedOpen}>▸</span>
        Advanced
      </button>
      {#if advancedOpen}
        <div class="advanced-body">
          <label>
            <span>Extra JVM arguments</span>
            <input
              type="text"
              bind:value={javaArgs}
              placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=200"
              spellcheck="false"
            />
          </label>
          <label>
            <span>Custom start command (overrides everything)</span>
            <input
              type="text"
              bind:value={startCommand}
              placeholder="java -Xmx4G -jar server.jar nogui"
              spellcheck="false"
            />
          </label>
          <p class="hint">
            The custom command runs from the server folder and replaces the java
            invocation entirely — memory and JVM args above are ignored.
          </p>
        </div>
      {/if}
    </div>

    {#if creating}
      <div class="progress">
        <ProgressBar value={progress} />
        <p class="hint">Downloading the server software… 📦</p>
      </div>
    {/if}

    <div class="actions">
      <Button type="submit" disabled={!canSubmit}>
        {creating ? "Creating…" : "Create server 🚀"}
      </Button>
    </div>
  </form>
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--muted);
  }

  .row {
    display: flex;
    gap: 0.75rem;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .port-label {
    width: 110px;
    flex-shrink: 0;
  }

  input[type="text"],
  input[type="number"],
  select {
    font-family: inherit;
    font-size: 1rem;
    color: var(--text);
    background: var(--surface-2);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    padding: 0.6em 0.9em;
    outline: none;
    transition: border-color 0.18s ease;
    min-width: 0;
  }

  input[type="text"]:focus,
  input[type="number"]:focus,
  select:focus {
    border-color: var(--accent);
  }

  input[type="range"] {
    accent-color: var(--accent);
  }

  .checkbox {
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }

  .checkbox input {
    width: 1.1rem;
    height: 1.1rem;
    accent-color: var(--accent);
  }

  .eula a {
    color: var(--accent);
  }

  .proxy-note {
    margin: 0;
    font-size: 0.82rem;
    color: var(--muted);
  }

  .loading {
    color: var(--muted);
    font-weight: 400;
    padding: 0.6em 0;
  }

  .location {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .location-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--muted);
  }

  .location-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .location-path {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--text);
    background: var(--surface-2);
    border-radius: var(--radius-md);
    padding: 0.6em 0.9em;
    overflow-wrap: break-word;
    word-break: break-all;
  }

  .advanced-toggle {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    border: none;
    background: transparent;
    color: var(--muted);
    font-family: inherit;
    font-size: 0.9rem;
    font-weight: 700;
    padding: 0;
    cursor: pointer;
  }

  .advanced-toggle:hover {
    color: var(--text);
  }

  .chevron {
    display: inline-block;
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .advanced-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.75rem;
  }

  .progress {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
    text-align: left;
  }

  .progress .hint {
    text-align: center;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
