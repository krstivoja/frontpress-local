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
        .manage(AppState::load())
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
            commands::get_credentials,
            commands::get_settings,
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
