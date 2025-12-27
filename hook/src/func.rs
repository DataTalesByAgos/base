use crate::offsets;
use crate::structs::{GameData, Item, InventorySection, CharacterHuman, KenshiString};
use crate::game_state; // assuming MODULE_BASE is accessible or passed
use std::mem::transmute;
use std::ffi::c_void;

pub unsafe fn spawn_item(item_info: *mut GameData) -> *mut Item {
    let module_base = game_state::MODULE_BASE;
    let spawn_item_func: extern "system" fn(
        *mut c_void, // rcx
        *mut GameData, // rdx
        *mut c_void, // r8
        *mut c_void, // r9
    ) -> *mut Item = transmute(module_base + offsets::SPAWN_ITEM_FUNC);

    let magic_ptr = (module_base + offsets::ITEM_SPAWNING_MAGIC) as *mut *mut c_void;
    let magic = *magic_ptr;
    let hand = (module_base + offsets::ITEM_SPAWNING_HAND) as *mut c_void;

    // The C++ code passes 7 args, but the last 3 are unused stack args.
    // In x64, the first 4 are registers (rcx, rdx, r8, r9).
    // If the function strictly expects stack space, Rust might handle it if we define enough args,
    // or we might trust the ABI.
    // C++ def: void* unused_stack1, void* unused_stack2, void* unused_stack3
    // Let's match the signature to be safe.
    let spawn_item_func_full: extern "system" fn(
        *mut c_void, *mut GameData, *mut c_void, *mut c_void,
        *mut c_void, *mut c_void, *mut c_void
    ) -> *mut Item = transmute(module_base + offsets::SPAWN_ITEM_FUNC);

    spawn_item_func_full(magic, item_info, hand, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut())
}

pub unsafe fn get_inv_section(npc: *mut CharacterHuman, section_name: *mut KenshiString) -> *mut InventorySection {
    let module_base = game_state::MODULE_BASE;
    let func: extern "system" fn(
        *mut c_void, // Inventory* inv (rcx)
        *mut KenshiString // section (rdx)
    ) -> *mut InventorySection = transmute(module_base + offsets::GET_SECTION_FROM_INV_BY_NAME);

    // C++: npc->inventory (rcx)
    func((*npc).inventory as *mut c_void, section_name)
}

// Generic caller for debug
pub unsafe fn call_dynamic(
    addr: usize, 
    a1: usize, a2: usize, a3: usize, a4: usize, 
    a5: usize, a6: usize, a7: usize
) -> usize {
    let func: extern "system" fn(
        usize, usize, usize, usize,
        usize, usize, usize
    ) -> usize = transmute(addr);
    
    func(a1, a2, a3, a4, a5, a6, a7)
}
