#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use yai_application::{LocalApplication, OperationRequest, OperationResult};

#[tauri::command]
fn studio_call(request: OperationRequest) -> OperationResult {
    LocalApplication::default().call(request)
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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![studio_call])
        .setup(move |app| {
            start_case_update_bridge(app.handle().clone(), running.clone());
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                shutdown.store(false, Ordering::Relaxed);
            }
        })
        .run(tauri::generate_context!())
        .expect("YAI Studio desktop bootstrap failed");
}
