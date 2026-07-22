fn main() {
    // The Windows window/taskbar icon and the macOS bundle icon are embedded
    // into the binary at compile time. Cargo does not otherwise treat the icon
    // files as build inputs, so editing them leaves a stale icon baked into an
    // already-compiled binary. Declare them explicitly so a regenerated icon
    // actually forces a rebuild.
    println!("cargo:rerun-if-changed=icons");

    tauri_build::build()
}
