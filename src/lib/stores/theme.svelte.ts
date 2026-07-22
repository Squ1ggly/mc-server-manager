// Colour-scheme preference: light, dark, or follow the OS.
//
// theme.css knows only two states — the light tokens on :root, the dark tokens
// on :root[data-theme="dark"]. "System" is resolved here rather than with a
// `prefers-color-scheme` media query so the dark palette stays defined in
// exactly one place and a manual choice can override the OS.

export type ThemePreference = "light" | "dark" | "system";

const THEME_PREFERENCES: ThemePreference[] = ["light", "dark", "system"];

const STORAGE_KEY = "serverforge:theme";
const DARK_SCHEME_QUERY = "(prefers-color-scheme: dark)";

function isThemePreference(value: string | null): value is ThemePreference {
  if (value === null) {
    return false;
  }
  const isKnown = THEME_PREFERENCES.includes(value as ThemePreference);
  return isKnown;
}

/** The stored choice, or "system" if there isn't a usable one. */
function readStoredPreference(): ThemePreference {
  // localStorage throws rather than returning null when storage is blocked,
  // so a failure here means "no stored choice", not a bug worth surfacing.
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (!isThemePreference(stored)) {
      return "system";
    }
    return stored;
  } catch {
    return "system";
  }
}

function storePreference(preference: ThemePreference): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, preference);
  } catch {
    // A preference we can't persist still applies for this session.
  }
}

function systemPrefersDark(): boolean {
  const prefersDark = window.matchMedia(DARK_SCHEME_QUERY).matches;
  return prefersDark;
}

class ThemeStore {
  private selected = $state<ThemePreference>("system");
  private systemIsDark = $state(false);

  get preference(): ThemePreference {
    return this.selected;
  }

  /** Which palette is actually showing, once "system" is resolved. */
  get isDark(): boolean {
    if (this.selected === "system") {
      return this.systemIsDark;
    }
    return this.selected === "dark";
  }

  /**
   * Applies the stored preference and keeps following the OS while the app
   * runs. Call once, before mounting, so the first paint is already correct.
   */
  start(): void {
    this.selected = readStoredPreference();
    this.systemIsDark = systemPrefersDark();
    this.apply();

    const darkSchemeQuery = window.matchMedia(DARK_SCHEME_QUERY);
    darkSchemeQuery.addEventListener("change", (event) => {
      this.systemIsDark = event.matches;
      this.apply();
    });
  }

  select(preference: ThemePreference): void {
    this.selected = preference;
    storePreference(preference);
    this.apply();
  }

  private apply(): void {
    const activeTheme = this.isDark ? "dark" : "light";
    document.documentElement.dataset.theme = activeTheme;
  }
}

export const themeStore = new ThemeStore();
