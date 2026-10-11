//! wasm32 entry point: trunk compiles this bin. The app lives in the lib
//! (`mimir_web::App`), so host builds check it.

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(mimir_web::App)
}
