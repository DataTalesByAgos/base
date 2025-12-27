use windows::Win32::Foundation::{HINSTANCE, BOOL, TRUE};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use std::thread;

mod memory;
mod hooks;
// mod overlay; // TODO: Implement overlay

#[no_mangle]
extern "system" fn DllMain(
    _hinstance: HINSTANCE,
    reason: u32,
    _reserved: *const std::ffi::c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                // Disable thread library calls for optimization if not needed
                windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls(_hinstance);
            }
            
            // Spawn a new thread to avoid blocking the loader lock
            thread::spawn(|| {
                main_thread();
            });
        }
        DLL_PROCESS_DETACH => {
            // Cleanup provided
        }
        _ => {}
    }
    TRUE
}

fn main_thread() {
    // Initialize console for debugging
    unsafe {
        windows::Win32::System::Console::AllocConsole();
    }
    println!("[KenshiOnline] Mod injected successfully!");

    // Initialize Hooks
    if let Err(e) = hooks::initialize() {
        eprintln!("[KenshiOnline] Failed to initialize hooks: {:?}", e);
        return;
    }

    // Initialize Network Client
    // connect_to_server();

    // Main loop
    loop {
        // Game logic updates
        thread::sleep(std::time::Duration::from_millis(100));
    }
}
