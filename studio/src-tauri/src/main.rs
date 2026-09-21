#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod terminal;

use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State, WebviewWindow};
use tauri_runtime::ResizeDirection;
use terminal::{PtyHost, TerminalCreated, TerminalEvents, TerminalExit, TerminalOutput};
use yai_application::{OperationError, OperationRequest, OperationResult, ResultState};
use yai_host::{ClientKind, HostClient, HostEvent, HostTelemetry};

#[derive(Clone)]
struct StudioHost {
    home: PathBuf,
    executable: PathBuf,
    explicitly_stopped: Arc<AtomicBool>,
    last_instance: Arc<Mutex<Option<String>>>,
}

impl StudioHost {
    fn new() -> Result<Self, String> {
        let home = std::env::var_os("YAI_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".yai")
            });
        Ok(Self {
            home,
            executable: std::env::current_exe()
                .map_err(|error| format!("studio_executable_unavailable:{error}"))?,
            explicitly_stopped: Arc::new(AtomicBool::new(false)),
            last_instance: Arc::new(Mutex::new(None)),
        })
    }

    fn ensure_started(&self) -> Result<HostTelemetry, String> {
        if self.explicitly_stopped.load(Ordering::Acquire) {
            return Err("host_explicitly_stopped".into());
        }
        let telemetry = yai_host::start(&self.home, &self.executable, &["--yai-local-host-serve"])?;
        Ok(telemetry)
    }

    fn remember(&self, telemetry: &HostTelemetry) {
        if let Ok(mut instance) = self.last_instance.lock() {
            *instance = telemetry.instance_id.clone();
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct HostConnectionEvent {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    telemetry: Option<HostTelemetry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    resync_required: bool,
}

#[tauri::command]
fn studio_call(host: State<'_, StudioHost>, request: OperationRequest) -> OperationResult {
    let result = host
        .ensure_started()
        .and_then(|_| HostClient::connect(&host.home, ClientKind::Studio))
        .and_then(|client| client.call(request.clone()));
    match result {
        Ok(result) => result,
        Err(error) => OperationResult {
            operation_ref: request.operation_ref,
            result_state: ResultState::TransportUnavailable,
            correlation_ref: request.correlation_ref,
            data: None,
            error: Some(OperationError {
                code: error.clone(),
                message: error,
                safe_message: "The resident local YAI Host is unavailable.".into(),
                result_state: ResultState::TransportUnavailable,
            }),
        },
    }
}

#[tauri::command]
fn studio_host_status(host: State<'_, StudioHost>) -> Result<HostTelemetry, String> {
    let telemetry = yai_host::observe(&host.home)?;
    host.remember(&telemetry);
    Ok(telemetry)
}

#[tauri::command]
fn studio_host_start(host: State<'_, StudioHost>) -> Result<HostTelemetry, String> {
    host.explicitly_stopped.store(false, Ordering::Release);
    let telemetry = host.ensure_started()?;
    host.remember(&telemetry);
    Ok(telemetry)
}

#[tauri::command]
fn studio_host_stop(host: State<'_, StudioHost>) -> Result<HostTelemetry, String> {
    host.explicitly_stopped.store(true, Ordering::Release);
    yai_host::stop(&host.home)
}

#[tauri::command]
fn studio_host_restart(host: State<'_, StudioHost>) -> Result<HostTelemetry, String> {
    let telemetry = yai_host::restart(&host.home, &host.executable, &["--yai-local-host-serve"])?;
    host.explicitly_stopped.store(false, Ordering::Release);
    host.remember(&telemetry);
    Ok(telemetry)
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

fn emit_host_state(
    app: &AppHandle,
    state: &str,
    telemetry: Option<HostTelemetry>,
    reason: Option<String>,
    resync_required: bool,
) {
    let _ = app.emit(
        "yai://host-state",
        HostConnectionEvent {
            state: state.into(),
            telemetry,
            reason,
            resync_required,
        },
    );
}

fn start_host_event_bridge(app: tauri::AppHandle, running: Arc<AtomicBool>, host: StudioHost) {
    std::thread::spawn(move || {
        let mut connected_once = false;
        while running.load(Ordering::Relaxed) {
            if host.explicitly_stopped.load(Ordering::Acquire) {
                match yai_host::observe(&host.home) {
                    Ok(telemetry) if telemetry.state == "running" => {
                        host.explicitly_stopped.store(false, Ordering::Release);
                        host.remember(&telemetry);
                    }
                    Ok(telemetry) => {
                        emit_host_state(&app, "stopped", Some(telemetry), None, false);
                        std::thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                    Err(error) => {
                        emit_host_state(&app, "unavailable", None, Some(error), false);
                        std::thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                }
            }
            let telemetry = match yai_host::observe(&host.home) {
                Ok(telemetry) if telemetry.state == "running" => telemetry,
                _ => {
                    emit_host_state(
                        &app,
                        if connected_once {
                            "reconnecting"
                        } else {
                            "starting"
                        },
                        None,
                        None,
                        connected_once,
                    );
                    match host.ensure_started() {
                        Ok(telemetry) => telemetry,
                        Err(error) => {
                            emit_host_state(&app, "unavailable", None, Some(error), connected_once);
                            std::thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                    }
                }
            };
            let prior = host
                .last_instance
                .lock()
                .ok()
                .and_then(|value| value.clone());
            let changed_instance = prior.as_deref() != telemetry.instance_id.as_deref();
            host.remember(&telemetry);
            emit_host_state(
                &app,
                "live",
                Some(telemetry),
                None,
                connected_once && changed_instance,
            );
            connected_once = true;
            let subscription =
                HostClient::connect(&host.home, ClientKind::Studio).and_then(|client| {
                    client.subscribe(|event| {
                        if !running.load(Ordering::Acquire) {
                            return Err("studio_client_closed".into());
                        }
                        match event {
                            HostEvent::Case(update) => {
                                let _ = app.emit("yai://case-update", update);
                            }
                            HostEvent::Heartbeat(telemetry) => {
                                host.remember(&telemetry);
                                emit_host_state(&app, "live", Some(telemetry), None, false);
                            }
                            HostEvent::Shutdown(reason) => {
                                host.explicitly_stopped.store(true, Ordering::Release);
                                emit_host_state(&app, "stopped", None, Some(reason), true);
                            }
                        }
                        Ok(())
                    })
                });
            if let Err(error) = subscription {
                emit_host_state(&app, "reconnecting", None, Some(error), true);
            }
        }
    });
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--yai-local-host-serve") {
        let home = std::env::var_os("YAI_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".yai")
            });
        std::process::exit(match yai_host::serve(home) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("YAI Local Host failed: {error}");
                1
            }
        });
    }
    let running = Arc::new(AtomicBool::new(true));
    let shutdown = running.clone();
    let terminal_host = PtyHost::default();
    let shutdown_terminals = terminal_host.clone();
    let studio_host = StudioHost::new().expect("YAI Studio Host lifecycle bootstrap failed");
    let event_host = studio_host.clone();
    let fixture_mode = std::env::var("VITE_STUDIO_MODE").ok().as_deref() == Some("fixture");
    tauri::Builder::default()
        .manage(terminal_host)
        .manage(studio_host)
        .invoke_handler(tauri::generate_handler![
            studio_call,
            studio_host_status,
            studio_host_start,
            studio_host_stop,
            studio_host_restart,
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
            if !fixture_mode {
                start_host_event_bridge(app.handle().clone(), running.clone(), event_host.clone());
            }
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
