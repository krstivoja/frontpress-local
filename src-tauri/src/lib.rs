mod commands;
mod frontpress;
mod keychain;
mod net;
mod paths;
mod php;
mod server;
mod store;
mod util;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::load())
        .setup(|app| {
            // Stamp the running version to disk so a self-update can be
            // verified out-of-band (the file flips to the new version after
            // the updater relaunches the app).
            if let Ok(dir) = crate::paths::app_data_dir() {
                let v = app.package_info().version.to_string();
                let _ = std::fs::write(dir.join("last-run-version.txt"), v);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::available_php,
            commands::install_php,
            commands::set_global_php,
            commands::create_site,
            commands::start_site,
            commands::stop_site,
            commands::delete_site,
            commands::open_preview,
            commands::auto_login,
            commands::reveal_in_finder,
            commands::get_settings,
            commands::selftest_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building FrontPress Local")
        .run(|app_handle, event| {
            // Make sure no orphan `php -S` processes survive the app.
            if let tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit = event {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    state.servers.stop_all();
                }
            }
        });
}
