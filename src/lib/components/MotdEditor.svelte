<script lang="ts">
  // MOTD editor in the style of the classic web MOTD creators: a text field
  // with §-code buttons and a live, game-style preview.

  import { MOTD_COLORS, parseMotd } from "../motd";

  interface Props {
    /** Editor-form text (real § characters). */
    value: string;
    onchange: (value: string) => void;
  }

  let { value, onchange }: Props = $props();

  let inputElement = $state<HTMLInputElement | null>(null);

  const FORMAT_BUTTONS = [
    { code: "l", label: "B", title: "Bold" },
    { code: "o", label: "I", title: "Italic" },
    { code: "n", label: "U", title: "Underline" },
    { code: "m", label: "S", title: "Strikethrough" },
    { code: "r", label: "⟲", title: "Reset formatting" },
  ];

  const previewSpans = $derived(parseMotd(value));

  function insertCode(code: string) {
    const insertion = `§${code}`;
    const cursor = inputElement?.selectionStart ?? value.length;
    const updated = value.slice(0, cursor) + insertion + value.slice(cursor);
    onchange(updated);

    // Put the cursor right after the inserted code.
    requestAnimationFrame(() => {
      inputElement?.focus();
      inputElement?.setSelectionRange(cursor + insertion.length, cursor + insertion.length);
    });
  }
</script>

<div class="motd-editor">
  <div class="toolbar">
    <span class="swatches">
      {#each MOTD_COLORS as colorEntry (colorEntry.code)}
        <button
          type="button"
          class="swatch"
          style:background={colorEntry.hex}
          title="{colorEntry.name} (§{colorEntry.code})"
          onclick={() => insertCode(colorEntry.code)}
          aria-label={colorEntry.name}
        ></button>
      {/each}
    </span>
    <span class="formats">
      {#each FORMAT_BUTTONS as format (format.code)}
        <button
          type="button"
          class="format"
          title="{format.title} (§{format.code})"
          onclick={() => insertCode(format.code)}
        >
          {format.label}
        </button>
      {/each}
    </span>
  </div>

  <input
    type="text"
    bind:this={inputElement}
    {value}
    oninput={(event) => onchange(event.currentTarget.value)}
    placeholder="§aA §lMinecraft §r§aServer"
    spellcheck="false"
  />

  <div class="preview">
    {#if previewSpans.length === 0}
      <span class="preview-empty">A Minecraft Server</span>
    {:else}
      {#each previewSpans as span, index (index)}
        <span
          style:color={span.color ?? "#AAAAAA"}
          class:bold={span.bold}
          class:italic={span.italic}
          class:underline={span.underline}
          class:strike={span.strike}>{span.text}</span
        >
      {/each}
    {/if}
  </div>
</div>

<style>
  .motd-editor {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .swatches {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
  }

  .swatch {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    box-shadow:
      inset 0 2px 0 rgba(255, 255, 255, 0.25),
      inset 0 -2px 0 rgba(0, 0, 0, 0.25),
      0 0 0 1px rgba(15, 15, 18, 0.4);
  }

  .formats {
    display: flex;
    gap: 3px;
  }

  .format {
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--text);
    font-family: var(--font-pixel);
    font-size: 0.85rem;
    cursor: pointer;
    box-shadow:
      inset 0 2px 0 rgba(255, 255, 255, 0.08),
      inset 0 -2px 0 rgba(0, 0, 0, 0.2);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .format:hover {
    background: var(--accent-soft);
  }

  input {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    color: var(--text);
    background: var(--surface-2);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    padding: 0.55em 0.8em;
    outline: none;
    transition: border-color 0.18s ease;
  }

  input:focus {
    border-color: var(--accent);
  }

  /* Server-list style preview: always dark, pixel font. */
  .preview {
    background: #1a1b1e;
    border-radius: var(--radius-sm);
    box-shadow: inset 0 2px 0 rgba(0, 0, 0, 0.5);
    padding: 0.6rem 0.9rem;
    font-family: var(--font-pixel);
    font-size: 0.95rem;
    color: #aaaaaa;
    min-height: 1.6em;
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .preview-empty {
    color: #55555c;
  }

  .bold {
    font-weight: 700;
  }

  .italic {
    font-style: italic;
  }

  .underline {
    text-decoration: underline;
  }

  .strike {
    text-decoration: line-through;
  }

  .underline.strike {
    text-decoration: underline line-through;
  }
</style>
