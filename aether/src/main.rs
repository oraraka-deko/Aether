use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    println!("Starting Aether Binary CLI Mode...");
    
    let args_str = args[1..].join(" ");
    
    unsafe {
        // Register the local CLI logger to stream FFI messages to console
        aether::aether_set_log_callback(Some(cli_log_callback));
        
        // Start the engine
        aether::aether_start(std::ffi::CString::new(args_str).unwrap().as_ptr());
        
        // Block the CLI thread while active
        while aether::aether_is_running() == 1 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

extern "C" fn cli_log_callback(msg: *const std::os::raw::c_char) {
    unsafe {
        if !msg.is_null() {
            if let Ok(c_str) = std::ffi::CStr::from_ptr(msg).to_str() {
                println!("[Aether] {}", c_str);
            }
        }
    }
}
