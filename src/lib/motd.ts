// MOTD (§-code) helpers: parsing for live preview, plus the escaping that
// server.properties needs (Java reads it as latin-1, so a raw UTF-8 '§'
// would be mangled — the file stores § escapes instead).

export const MOTD_COLORS: { code: string; name: string; hex: string }[] = [
  { code: "0", name: "Black", hex: "#000000" },
  { code: "1", name: "Dark Blue", hex: "#0000AA" },
  { code: "2", name: "Dark Green", hex: "#00AA00" },
  { code: "3", name: "Dark Aqua", hex: "#00AAAA" },
  { code: "4", name: "Dark Red", hex: "#AA0000" },
  { code: "5", name: "Dark Purple", hex: "#AA00AA" },
  { code: "6", name: "Gold", hex: "#FFAA00" },
  { code: "7", name: "Gray", hex: "#AAAAAA" },
  { code: "8", name: "Dark Gray", hex: "#555555" },
  { code: "9", name: "Blue", hex: "#5555FF" },
  { code: "a", name: "Green", hex: "#55FF55" },
  { code: "b", name: "Aqua", hex: "#55FFFF" },
  { code: "c", name: "Red", hex: "#FF5555" },
  { code: "d", name: "Light Purple", hex: "#FF55FF" },
  { code: "e", name: "Yellow", hex: "#FFFF55" },
  { code: "f", name: "White", hex: "#FFFFFF" },
];

export interface MotdSpan {
  text: string;
  color?: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
}

/** Parses §-coded text into styled spans for the preview. */
export function parseMotd(text: string): MotdSpan[] {
  const spans: MotdSpan[] = [];
  let current: MotdSpan = { text: "", bold: false, italic: false, underline: false, strike: false };

  const flush = () => {
    if (current.text !== "") {
      spans.push(current);
    }
    current = { ...current, text: "" };
  };

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (char !== "§") {
      current.text += char;
      continue;
    }

    const code = text[i + 1]?.toLowerCase();
    i++;
    const colorEntry = MOTD_COLORS.find((entry) => entry.code === code);
    flush();
    if (colorEntry) {
      // A color code resets formatting, exactly like the game does.
      current = {
        text: "",
        color: colorEntry.hex,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
      };
    } else if (code === "l") {
      current.bold = true;
    } else if (code === "o") {
      current.italic = true;
    } else if (code === "n") {
      current.underline = true;
    } else if (code === "m") {
      current.strike = true;
    } else if (code === "r") {
      current = { text: "", bold: false, italic: false, underline: false, strike: false };
    }
  }

  flush();
  return spans;
}

/** server.properties form (§ escapes, literal \n) -> editor form. */
export function decodeMotdProperty(stored: string): string {
  return stored.replaceAll("\\u00A7", "§").replaceAll("\\u00a7", "§");
}

/** Editor form -> server.properties form. */
export function encodeMotdProperty(editorText: string): string {
  return editorText.replaceAll("§", "\\u00A7");
}
