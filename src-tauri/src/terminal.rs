use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Window};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub id: String,
    pub title: String,
    pub rows: u16,
    pub cols: u16,
}

pub struct PtySession {
    pub _id: String,
    pub pty_pair: PtyPair,
    pub _child: Box<dyn portable_pty::Child + Send + Sync>,
    pub writer: Box<dyn Write + Send>,
    pub shutdown_tx: mpsc::Sender<()>,
}

#[derive(Clone)]
pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, Arc<Mutex<PtySession>>>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_terminal(&self, id: &str) -> Option<Arc<Mutex<PtySession>>> {
        self.terminals.lock().unwrap().get(id).cloned()
    }

    pub fn add_terminal(&self, id: String, session: PtySession) {
        self.terminals
            .lock()
            .unwrap()
            .insert(id, Arc::new(Mutex::new(session)));
    }

    pub fn remove_terminal(&self, id: &str) -> Option<Arc<Mutex<PtySession>>> {
        self.terminals.lock().unwrap().remove(id)
    }

    pub fn list_terminals(&self) -> Vec<TerminalInfo> {
        self.terminals
            .lock()
            .unwrap()
            .iter()
            .map(|(id, _)| TerminalInfo {
                id: id.clone(),
                title: format!("Terminal {}", &id[..8]),
                rows: 24,
                cols: 80,
            })
            .collect()
    }
}

#[tauri::command]
pub async fn spawn_terminal(
    window: Window,
    cols: Option<u16>,
    rows: Option<u16>,
    state: State<'_, TerminalManager>,
) -> Result<String, String> {
    let terminal_id = Uuid::new_v4().to_string();
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);

    let pty_system = native_pty_system();
    let pty_pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new("zsh");
    cmd.arg("-i"); // Interactive shell
    cmd.env("TERM", "xterm-256color");
    cmd.env("LC_ALL", "en_US.UTF-8");
    
    // Copy current environment variables
    for (key, value) in std::env::vars() {
        if key != "TERM" && key != "LC_ALL" {
            cmd.env(key, value);
        }
    }
    
    let child = pty_pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let reader = pty_pair
        .master
        .try_clone_reader()
        .map_err(|e| e.to_string())?;
    let writer = pty_pair
        .master
        .take_writer()
        .map_err(|e| e.to_string())?;

    let (shutdown_tx, _shutdown_rx) = mpsc::channel::<()>(1);

    let session = PtySession {
        _id: terminal_id.clone(),
        pty_pair,
        _child: child,
        writer,
        shutdown_tx,
    };

    state.add_terminal(terminal_id.clone(), session);
    
    // Start reading in a separate thread
    let terminal_id_for_read = terminal_id.clone();
    let window_for_read = window.clone();
    
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = vec![0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    window_for_read.emit(&format!("terminal-closed-{}", terminal_id_for_read), ()).ok();
                    break;
                }
                Ok(n) => {
                    let data = buffer[..n].to_vec();
                    window_for_read.emit(&format!("terminal-output-{}", terminal_id_for_read), data).ok();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    window_for_read.emit(&format!("terminal-error-{}", terminal_id_for_read), e.to_string()).ok();
                    break;
                }
            }
        }
    });

    Ok(terminal_id)
}

#[tauri::command]
pub async fn write_to_terminal(
    session_id: String,
    data: Vec<u8>,
    state: State<'_, TerminalManager>,
) -> Result<(), String> {
    if let Some(terminal_arc) = state.get_terminal(&session_id) {
        let mut terminal = terminal_arc.lock().unwrap();
        terminal
            .writer
            .write_all(&data)
            .map_err(|e| e.to_string())?;
        terminal.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Terminal session not found".to_string())
    }
}

#[tauri::command]
pub async fn resize_terminal(
    session_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, TerminalManager>,
) -> Result<(), String> {
    if let Some(terminal_arc) = state.get_terminal(&session_id) {
        let terminal = terminal_arc.lock().unwrap();
        terminal
            .pty_pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Terminal session not found".to_string())
    }
}

#[tauri::command]
pub async fn close_terminal(
    session_id: String,
    state: State<'_, TerminalManager>,
) -> Result<(), String> {
    if let Some(terminal_arc) = state.remove_terminal(&session_id) {
        let terminal = terminal_arc.lock().unwrap();
        terminal.shutdown_tx.try_send(()).ok();
        Ok(())
    } else {
        Err("Terminal session not found".to_string())
    }
}

#[tauri::command]
pub async fn list_terminals(state: State<'_, TerminalManager>) -> Result<Vec<TerminalInfo>, String> {
    Ok(state.list_terminals())
}

#[tauri::command]
pub async fn spawn_terminal_with_command(
    window: Window,
    cols: Option<u16>,
    rows: Option<u16>,
    working_dir: String,
    command: String,
    args: Vec<String>,
    state: State<'_, TerminalManager>,
) -> Result<String, String> {
    let terminal_id = Uuid::new_v4().to_string();
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);

    let pty_system = native_pty_system();
    let pty_pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new(command);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("LC_ALL", "en_US.UTF-8");
    cmd.cwd(working_dir);
    
    // Copy current environment variables
    for (key, value) in std::env::vars() {
        if key != "TERM" && key != "LC_ALL" {
            cmd.env(key, value);
        }
    }
    
    let child = pty_pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let reader = pty_pair
        .master
        .try_clone_reader()
        .map_err(|e| e.to_string())?;
    let writer = pty_pair
        .master
        .take_writer()
        .map_err(|e| e.to_string())?;

    let (shutdown_tx, _shutdown_rx) = mpsc::channel::<()>(1);

    let session = PtySession {
        _id: terminal_id.clone(),
        pty_pair,
        _child: child,
        writer,
        shutdown_tx,
    };

    state.add_terminal(terminal_id.clone(), session);
    
    // Start reading in a separate thread
    let terminal_id_for_read = terminal_id.clone();
    let window_for_read = window.clone();
    
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = vec![0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    window_for_read.emit(&format!("terminal-closed-{}", terminal_id_for_read), ()).ok();
                    break;
                }
                Ok(n) => {
                    let data = buffer[..n].to_vec();
                    window_for_read.emit(&format!("terminal-output-{}", terminal_id_for_read), data).ok();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    window_for_read.emit(&format!("terminal-error-{}", terminal_id_for_read), e.to_string()).ok();
                    break;
                }
            }
        }
    });

    Ok(terminal_id)
}