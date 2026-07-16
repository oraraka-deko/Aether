// Declare existing internal modules to include in the library compilation
pub mod account;
pub mod aethernoize;
pub mod cli;
pub mod config;
pub mod consts;
pub mod dns;
pub mod error;
pub mod fragment;
pub mod lastconn;
pub mod mac_test;
pub mod masque;
pub mod masque_h2;
pub mod netstack;
pub mod noize;
pub mod prober;
pub mod quic;
pub mod socks;
pub mod tls;
pub mod wg_prober;
pub mod wireguard;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

// Thread-safe state tracking for the background runtime worker
static IS_RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref SHUTDOWN_TX: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);
    static ref LOG_CALLBACK: Mutex<Option<unsafe extern "C" fn(*const c_char)>> = Mutex::new(None);
}

/// Registers a callback function to stream logs and progress events back to the UI.
/// Used directly by Flutter (dart:ffi) or native platform wrappers.
#[no_mangle]
pub unsafe extern "C" fn aether_set_log_callback(callback: Option<unsafe extern "C" fn(*const c_char)>) {
    if let Ok(mut guard) = LOG_CALLBACK.lock() {
        *guard = callback;
    }
}

/// Dispatches raw text back to the host application's log listener
pub fn log_message(msg: &str) {
    if let Ok(guard) = LOG_CALLBACK.lock() {
        if let Some(callback) = *guard {
            if let Ok(c_msg) = CString::new(msg) {
                unsafe { callback(c_msg.as_ptr()) };
            }
        }
    }
}

/// Returns 1 if the Aether core worker is running, otherwise 0.
#[no_mangle]
pub extern "C" fn aether_is_running() -> c_int {
    if IS_RUNNING.load(Ordering::SeqCst) {
        1
    } else {
        0
    }
}

/// Starts the Aether engine on a dedicated, non-blocking background thread.
/// Arguments should be formatted like standard CLI inputs (e.g. "--protocol masque --socks-port 10808")
#[no_mangle]
pub unsafe extern "C" fn aether_start(args_str: *const c_char) -> c_int {
    if IS_RUNNING.load(Ordering::SeqCst) {
        log_message("Warning: Aether is already active.");
        return -1;
    }

    if args_str.is_null() {
        log_message("Error: Received null configurations string pointer.");
        return -2;
    }

    let c_str = CStr::from_ptr(args_str);
    let args_string = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            log_message("Error: Arguments string is not valid UTF-8.");
            return -3;
        }
    };

    IS_RUNNING.store(true, Ordering::SeqCst);

    let (tx, rx) = oneshot::channel::<()>();
    if let Ok(mut guard) = SHUTDOWN_TX.lock() {
        *guard = Some(tx);
    }

    // Spawn an OS thread to prevent blocking the host UI/event loop
    thread::spawn(move || {
        log_message("Aether background thread spawned successfully.");

        let rt = match Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                log_message(&format!("Fatal: Failed to construct Tokio runtime: {:?}", e));
                IS_RUNNING.store(false, Ordering::SeqCst);
                return;
            }
        };

        rt.block_on(async {
            let parsed_args: Vec<String> = args_string
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            tokio::select! {
                _ = rx => {
                    log_message("Aether tunnel received manual shutdown command.");
                }
                res = run_tunnel_core(parsed_args) => {
                    if let Err(e) = res {
                        log_message(&format!("Aether loop exited with error: {:?}", e));
                    }
                }
            }
        });

        IS_RUNNING.store(false, Ordering::SeqCst);
        log_message("Aether background thread has exited.");
    });

    0
}

/// Cleanly stops the running Aether background worker.
#[no_mangle]
pub extern "C" fn aether_stop() -> c_int {
    if !IS_RUNNING.load(Ordering::SeqCst) {
        log_message("Aether is not running.");
        return -1;
    }

    if let Ok(mut guard) = SHUTDOWN_TX.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }

    IS_RUNNING.store(false, Ordering::SeqCst);
    0
}

/// Internal asynchronous execution core
async fn run_tunnel_core(args: Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log_message("Parsing client credentials and resolving endpoints...");
    
    // Connects CLI arguments with internal configs
    // Example layout pointing to your existing CLI module:
    // let parsed_cfg = cli::parse_arguments(args)?;
    // parsed_cfg.run().await?;

    log_message(&format!("Core initialized with configurations: {:?}", args));

    // Replace the infinite keep-alive simulation block below with your actual main async runner loop
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
