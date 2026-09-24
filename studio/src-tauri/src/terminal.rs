use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MIN_COLS: u16 = 2;
const MIN_ROWS: u16 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct TerminalCreated {
    pub terminal_id: String,
    pub shell: String,
    pub cwd: String,
    pub pid: Option<u32>,
    pub created_at_unix_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TerminalSnapshot {
    pub studio_pid: u32,
    pub observed_at_unix_ms: u64,
    pub terminals: Vec<TerminalCreated>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64).unwrap_or(0)
}

#[derive(Clone, Debug, Serialize)]
pub struct TerminalOutput {
    pub terminal_id: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TerminalExit {
    pub terminal_id: String,
    pub exit_code: u32,
    pub signal: Option<String>,
}

pub trait TerminalEvents: Send + Sync + 'static {
    fn output(&self, payload: TerminalOutput);
    fn exited(&self, payload: TerminalExit);
}

struct TerminalSession {
    metadata: TerminalCreated,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

#[derive(Clone, Default)]
pub struct PtyHost {
    next_id: Arc<AtomicU64>,
    sessions: Arc<Mutex<BTreeMap<String, TerminalSession>>>,
}

impl PtyHost {
    pub fn create(
        &self,
        rows: u16,
        cols: u16,
        events: Arc<dyn TerminalEvents>,
    ) -> Result<TerminalCreated, String> {
        let rows = rows.max(MIN_ROWS);
        let cols = cols.max(MIN_COLS);
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("pty_open_failed: {error}"))?;

        let (shell_path, shell_name) = default_shell();
        let cwd = default_cwd();
        let mut command = CommandBuilder::new(&shell_path);
        command.cwd(&cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");

        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| format!("shell_start_failed: {error}"))?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("pty_reader_failed: {error}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| format!("pty_writer_failed: {error}"))?;
        let killer = child.clone_killer();
        let terminal_id = format!(
            "terminal:{}",
            self.next_id.fetch_add(1, Ordering::Relaxed) + 1
        );
        let metadata = TerminalCreated {
            terminal_id: terminal_id.clone(),
            shell: shell_name,
            cwd: cwd.to_string_lossy().into_owned(),
            pid: child.process_id(),
            created_at_unix_ms: now_ms(),
        };

        self.sessions
            .lock()
            .map_err(|_| "terminal_host_poisoned".to_string())?
            .insert(
                terminal_id.clone(),
                TerminalSession {
                    metadata: metadata.clone(),
                    master: pair.master,
                    writer,
                    killer,
                },
            );

        let output_id = terminal_id.clone();
        let output_events = events.clone();
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 16 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => output_events.output(TerminalOutput {
                        terminal_id: output_id.clone(),
                        data: buffer[..read].to_vec(),
                    }),
                }
            }
        });

        let exit_id = terminal_id.clone();
        let exit_events = events;
        let sessions = self.sessions.clone();
        std::thread::spawn(move || {
            let (exit_code, signal) = match child.wait() {
                Ok(status) => (status.exit_code(), status.signal().map(str::to_owned)),
                Err(_) => (1, Some("wait_failed".to_string())),
            };
            if let Ok(mut sessions) = sessions.lock() {
                sessions.remove(&exit_id);
            }
            exit_events.exited(TerminalExit {
                terminal_id: exit_id,
                exit_code,
                signal,
            });
        });

        Ok(metadata)
    }

    /// Only PTYs owned by this desktop process. No OS scan or Case projection.
    pub fn snapshot(&self) -> Result<TerminalSnapshot, String> {
        let sessions = self.sessions.lock().map_err(|_| "terminal_host_poisoned".to_string())?;
        Ok(TerminalSnapshot {
            studio_pid: std::process::id(),
            observed_at_unix_ms: now_ms(),
            terminals: sessions.values().map(|session| session.metadata.clone()).collect(),
        })
    }

    pub fn write(&self, terminal_id: &str, data: &[u8]) -> Result<(), String> {
        validate_terminal_id(terminal_id)?;
        if data.len() > MAX_INPUT_BYTES {
            return Err("terminal_input_too_large".to_string());
        }
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "terminal_host_poisoned".to_string())?;
        let session = sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal_not_found".to_string())?;
        session
            .writer
            .write_all(data)
            .and_then(|_| session.writer.flush())
            .map_err(|error| format!("terminal_write_failed: {error}"))
    }

    pub fn resize(&self, terminal_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        validate_terminal_id(terminal_id)?;
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| "terminal_host_poisoned".to_string())?;
        let session = sessions
            .get(terminal_id)
            .ok_or_else(|| "terminal_not_found".to_string())?;
        session
            .master
            .resize(PtySize {
                rows: rows.max(MIN_ROWS),
                cols: cols.max(MIN_COLS),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("terminal_resize_failed: {error}"))
    }

    pub fn kill(&self, terminal_id: &str) -> Result<(), String> {
        validate_terminal_id(terminal_id)?;
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "terminal_host_poisoned".to_string())?;
        let session = sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal_not_found".to_string())?;
        session
            .killer
            .kill()
            .map_err(|error| format!("terminal_kill_failed: {error}"))
    }

    pub fn kill_all(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for session in sessions.values_mut() {
                let _ = session.killer.kill();
            }
            sessions.clear();
        }
    }

    #[cfg(test)]
    fn count(&self) -> usize {
        self.sessions
            .lock()
            .map(|sessions| sessions.len())
            .unwrap_or(0)
    }
}

fn validate_terminal_id(terminal_id: &str) -> Result<(), String> {
    let suffix = terminal_id
        .strip_prefix("terminal:")
        .ok_or_else(|| "invalid_terminal_id".to_string())?;
    if suffix.is_empty() || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("invalid_terminal_id".to_string());
    }
    Ok(())
}

fn default_cwd() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn default_shell() -> (PathBuf, String) {
    #[cfg(windows)]
    let candidate = std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into());
    #[cfg(not(windows))]
    let candidate = std::env::var_os("SHELL").unwrap_or_else(|| "/bin/sh".into());
    let path = PathBuf::from(candidate);
    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("shell")
        .to_string();
    (path, label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{self, Receiver};
    use std::time::{Duration, Instant};

    struct TestEvents {
        output: mpsc::Sender<TerminalOutput>,
        exit: mpsc::Sender<TerminalExit>,
    }

    impl TerminalEvents for TestEvents {
        fn output(&self, payload: TerminalOutput) {
            let _ = self.output.send(payload);
        }
        fn exited(&self, payload: TerminalExit) {
            let _ = self.exit.send(payload);
        }
    }

    fn harness() -> (
        PtyHost,
        Arc<TestEvents>,
        Receiver<TerminalOutput>,
        Receiver<TerminalExit>,
    ) {
        let (output_tx, output_rx) = mpsc::channel();
        let (exit_tx, exit_rx) = mpsc::channel();
        (
            PtyHost::default(),
            Arc::new(TestEvents {
                output: output_tx,
                exit: exit_tx,
            }),
            output_rx,
            exit_rx,
        )
    }

    fn wait_for_text(receiver: &Receiver<TerminalOutput>, needle: &str) -> Option<String> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut bytes = Vec::new();
        while Instant::now() < deadline {
            if let Ok(payload) = receiver.recv_timeout(Duration::from_millis(100)) {
                bytes.extend(payload.data);
                let text = String::from_utf8_lossy(&bytes);
                if text.contains(needle) {
                    return Some(text.into_owned());
                }
            }
        }
        None
    }

    #[test]
    fn creates_writes_resizes_and_exits() {
        let (host, events, output, exit) = harness();
        let terminal = host.create(24, 80, events).expect("create terminal");
        host.resize(&terminal.terminal_id, 31, 97)
            .expect("resize terminal");
        std::thread::sleep(Duration::from_secs(1));
        host.write(&terminal.terminal_id, b"stty -echo\n")
            .expect("disable input echo");
        std::thread::sleep(Duration::from_millis(100));
        host.write(&terminal.terminal_id, b"stty size\n")
            .expect("write terminal");
        let output = wait_for_text(&output, "31 97").expect("terminal output");
        assert!(
            output.contains("31 97"),
            "resize did not reach PTY: {output:?}"
        );
        host.write(&terminal.terminal_id, b"exit 7\n")
            .expect("exit shell");
        let exited = exit
            .recv_timeout(Duration::from_secs(5))
            .expect("exit event");
        assert_eq!(exited.terminal_id, terminal.terminal_id);
        assert_eq!(exited.exit_code, 7);
    }

    #[test]
    fn supports_multiple_terminals_kill_and_cleanup() {
        let (host, events, _output, exit) = harness();
        let first = host.create(24, 80, events.clone()).expect("first terminal");
        let second = host.create(24, 80, events).expect("second terminal");
        assert_eq!(host.count(), 2);
        let snapshot = host.snapshot().expect("shell snapshot");
        assert_eq!(snapshot.studio_pid, std::process::id());
        assert_eq!(snapshot.terminals.len(), 2);
        assert!(first.pid.is_some());
        assert_ne!(first.pid, second.pid);
        assert!(first.created_at_unix_ms > 0);
        assert!(snapshot.observed_at_unix_ms >= second.created_at_unix_ms);
        assert!(snapshot.terminals.iter().any(|item|
            item.terminal_id == first.terminal_id && item.pid == first.pid));
        host.kill(&first.terminal_id).expect("kill first");
        let _ = exit
            .recv_timeout(Duration::from_secs(5))
            .expect("first exit");
        let remaining = host.snapshot().expect("snapshot after exit");
        assert_eq!(remaining.terminals.len(), 1);
        assert_eq!(remaining.terminals[0].terminal_id, second.terminal_id);
        host.kill_all();
        let _ = exit
            .recv_timeout(Duration::from_secs(5))
            .expect("second exit");
        assert_eq!(host.count(), 0);
        assert!(host.snapshot().unwrap().terminals.is_empty());
        assert_ne!(first.terminal_id, second.terminal_id);
    }

    #[test]
    fn rejects_invalid_or_unknown_terminal_ids() {
        let host = PtyHost::default();
        assert_eq!(
            host.write("../../shell", b"x").unwrap_err(),
            "invalid_terminal_id"
        );
        assert_eq!(
            host.resize("terminal:404", 24, 80).unwrap_err(),
            "terminal_not_found"
        );
        assert_eq!(
            host.kill("terminal:nope").unwrap_err(),
            "invalid_terminal_id"
        );
    }

    #[test]
    #[cfg(unix)]
    fn hosts_fullscreen_tools_and_yai_cli_as_unparsed_bytes() {
        let has = |program: &str| {
            std::process::Command::new("sh")
                .args(["-c", &format!("command -v {program}")])
                .status()
                .is_ok_and(|status| status.success())
        };

        if has("vim") {
            let (host, events, output, _exit) = harness();
            let terminal = host.create(40, 120, events).expect("vim terminal");
            std::thread::sleep(Duration::from_secs(1));
            host.write(
                &terminal.terminal_id,
                b"vim -Nu NONE -n /tmp/yai-studio-pty-proof.txt\n",
            )
            .expect("launch vim");
            let vim = wait_for_text(&output, "\u{1b}[?1049h").expect("vim alternate screen");
            assert!(
                vim.contains("\u{1b}["),
                "vim emitted no terminal control bytes"
            );
            host.write(&terminal.terminal_id, b"\x1b:q!\n")
                .expect("exit vim");
            host.kill_all();
        }

        if has("less") {
            let (host, events, output, _exit) = harness();
            let terminal = host.create(40, 120, events).expect("less terminal");
            std::thread::sleep(Duration::from_secs(1));
            host.write(&terminal.terminal_id, b"seq 1 100 | less\n")
                .expect("launch less");
            let less = wait_for_text(&output, "\u{1b}[?1049h").expect("less alternate screen");
            assert!(
                less.contains("\u{1b}["),
                "less emitted no terminal control bytes"
            );
            host.write(&terminal.terminal_id, b"q").expect("exit less");
            host.kill_all();
        }

        let yai = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../yai");
        if yai.is_file() {
            let (host, events, output, _exit) = harness();
            let terminal = host.create(40, 120, events).expect("yai terminal");
            std::thread::sleep(Duration::from_secs(1));
            host.write(
                &terminal.terminal_id,
                format!("{} help\n", yai.display()).as_bytes(),
            )
            .expect("run yai help");
            let help = wait_for_text(&output, "YAI — governed operational AI runtime")
                .expect("yai help output");
            assert!(help.contains("Usage: yai <command>"));
            host.kill_all();
        }
    }
}
