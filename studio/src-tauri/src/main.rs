#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod terminal;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State, WebviewWindow};
use tauri_runtime::ResizeDirection;
use terminal::{PtyHost, TerminalCreated, TerminalEvents, TerminalExit, TerminalOutput};
use yai_application::{LocalApplication, OperationRequest, OperationResult};

#[tauri::command]
fn studio_call(request: OperationRequest) -> OperationResult {
    LocalApplication::default().call(request)
}

struct TauriTerminalEvents(AppHandle);

impl TerminalEvents for TauriTerminalEvents {
    fn output(&self, payload: TerminalOutput) {
        let _ = self.0.emit("yai://terminal-output", payload);
    }

    fn exited(&self, payload: TerminalExit) {
        let _ = self.0.emit("yai://terminal-exit", payload);
    }
}

#[tauri::command]
fn terminal_create(
    app: AppHandle,
    host: State<'_, PtyHost>,
    rows: u16,
    cols: u16,
) -> Result<TerminalCreated, String> {
    host.create(rows, cols, Arc::new(TauriTerminalEvents(app)))
}

#[tauri::command]
fn terminal_write(
    host: State<'_, PtyHost>,
    terminal_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    host.write(&terminal_id, &data)
}

#[tauri::command]
fn terminal_resize(
    host: State<'_, PtyHost>,
    terminal_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    host.resize(&terminal_id, rows, cols)
}

#[tauri::command]
fn terminal_kill(host: State<'_, PtyHost>, terminal_id: String) -> Result<(), String> {
    host.kill(&terminal_id)
}

#[tauri::command]
fn terminal_dispose_all(host: State<'_, PtyHost>) {
    host.kill_all();
}

#[tauri::command]
fn desktop_close(window: WebviewWindow) -> Result<(), String> {
    window
        .close()
        .map_err(|error| format!("desktop_close_failed: {error}"))
}

#[tauri::command]
fn desktop_minimize(window: WebviewWindow) -> Result<(), String> {
    window
        .minimize()
        .map_err(|error| format!("desktop_minimize_failed: {error}"))
}

#[tauri::command]
fn desktop_toggle_maximize(window: WebviewWindow) -> Result<(), String> {
    if window
        .is_maximized()
        .map_err(|error| format!("desktop_maximize_state_failed: {error}"))?
    {
        window
            .unmaximize()
            .map_err(|error| format!("desktop_restore_failed: {error}"))
    } else {
        window
            .maximize()
            .map_err(|error| format!("desktop_maximize_failed: {error}"))
    }
}

#[tauri::command]
fn desktop_start_dragging(window: WebviewWindow) -> Result<(), String> {
    window
        .start_dragging()
        .map_err(|error| format!("desktop_drag_failed: {error}"))
}

fn resize_direction(direction: &str) -> Result<ResizeDirection, String> {
    match direction {
        "north" => Ok(ResizeDirection::North),
        "north-east" => Ok(ResizeDirection::NorthEast),
        "east" => Ok(ResizeDirection::East),
        "south-east" => Ok(ResizeDirection::SouthEast),
        "south" => Ok(ResizeDirection::South),
        "south-west" => Ok(ResizeDirection::SouthWest),
        "west" => Ok(ResizeDirection::West),
        "north-west" => Ok(ResizeDirection::NorthWest),
        _ => Err(format!("desktop_resize_direction_invalid: {direction}")),
    }
}

#[tauri::command]
fn desktop_start_resize_dragging(window: tauri::Window, direction: String) -> Result<(), String> {
    window
        .start_resize_dragging(resize_direction(&direction)?)
        .map_err(|error| format!("desktop_resize_failed: {error}"))
}

fn start_case_update_bridge(app: tauri::AppHandle, running: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let application = LocalApplication::default();
        let mut generations = BTreeMap::new();
        let mut sequence = 0_u64;
        let mut initialized = false;
        while running.load(Ordering::Relaxed) {
            if let Ok(current) = application.visible_generations() {
                for (case_ref, generation) in &current {
                    let changed = generations
                        .get(case_ref)
                        .is_some_and(|known| known != generation);
                    let newly_visible = initialized && !generations.contains_key(case_ref);
                    if changed || newly_visible {
                        sequence = sequence.saturating_add(1);
                        let _ = app.emit(
                            "yai://case-update",
                            application.update_for(case_ref, *generation, sequence),
                        );
                    }
                }
                generations = current;
                initialized = true;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let shutdown = running.clone();
    let terminal_host = PtyHost::default();
    let shutdown_terminals = terminal_host.clone();
    tauri::Builder::default()
        .manage(terminal_host)
        .invoke_handler(tauri::generate_handler![
            studio_call,
            terminal_create,
            terminal_write,
            terminal_resize,
            terminal_kill,
            terminal_dispose_all,
            desktop_close,
            desktop_minimize,
            desktop_toggle_maximize,
            desktop_start_dragging,
            desktop_start_resize_dragging
        ])
        .setup(move |app| {
            start_case_update_bridge(app.handle().clone(), running.clone());
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                shutdown.store(false, Ordering::Relaxed);
                shutdown_terminals.kill_all();
            }
        })
        .run(tauri::generate_context!())
        .expect("YAI Studio desktop bootstrap failed");
}

#[cfg(test)]
mod window_tests {
    use super::resize_direction;

    #[test]
    fn resize_directions_are_strictly_bounded() {
        for direction in [
            "north",
            "north-east",
            "east",
            "south-east",
            "south",
            "south-west",
            "west",
            "north-west",
        ] {
            assert!(resize_direction(direction).is_ok());
        }
        assert!(resize_direction("center").is_err());
        assert!(resize_direction("north; run something").is_err());
    }
}
