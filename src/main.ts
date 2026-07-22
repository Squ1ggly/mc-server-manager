import { mount } from "svelte";
import App from "./App.svelte";
import { themeStore } from "./lib/stores/theme.svelte";
import "./lib/theme.css";

// Before mounting, so the first paint uses the right palette instead of
// flashing the light default.
themeStore.start();

const target = document.getElementById("app");
if (target === null) {
  throw new Error("missing #app mount point in index.html");
}

const app = mount(App, { target });

export default app;
