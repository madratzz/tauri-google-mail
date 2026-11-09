fn main() {
    // Declare the app's own commands so capabilities can grant them
    // individually (e.g. `allow-notify` for the remote Gmail origin).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["notify"])),
    )
    .expect("failed to run tauri build script");
}
