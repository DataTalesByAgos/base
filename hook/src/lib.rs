use std::ffi::c_void;
use windows::Win32::Foundation::{BOOL, HMODULE};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

pub mod offsets;
pub mod structs;
pub mod utils;
pub mod game_state;
pub mod func;
pub mod commands;
pub mod hooks;
pub mod network;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HMODULE,
    call_reason: u32,
    reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            // Spawn a thread to initialize the mod to avoid blocking DllMain
            std::thread::spawn(|| {
                unsafe { initialize() };
            });
        }
        DLL_PROCESS_DETACH => {
            // Cleanup provided by OS mostly
        }
        _ => {}
    }
    BOOL::from(true)
}

unsafe fn initialize() {
    utils::spawn_console();
    println!("Console Initialized");
    
    // Scan heap and init global pointers
    // Note: C++ does scanHeap() later, but we need GAME_WORLD set for many things.
    // game_state::init() finds MODULE_BASE and GAME_WORLD.
    game_state::init();
    println!("Game State Initialized");

    // Hooks
    hooks::setup_hooks();
    println!("Hooks Installed");

    // Spawn Commands Thread
    std::thread::spawn(|| {
        commands::commands_loop();
    });

    // Spawn Network Thread
    // Network is blocking, so own thread.
    std::thread::spawn(|| {
        network::connect_and_run();
    });
}
