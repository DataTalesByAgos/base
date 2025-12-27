use crate::utils::{create_hook, asmb};
use crate::offsets;
use crate::game_state;
use std::ffi::c_void;

pub unsafe fn setup_hooks() {
    let module_base = game_state::MODULE_BASE;

    // Char Update Hook
    let char_update_old_code: Vec<u8> = vec![
        0x48, 0x8B, 0x8B, 0x20, 0x03, 0x00, 0x00, // mov rcx,[rbx+00000320]
        0x40, 0x88, 0xB3, 0x7C, 0x03, 0x00, 0x00  // mov [rbx+0000037C],sil
    ];
    // To ensure on_char_update(rcx) gets the right pointer, we run the mov rcx instruction before calling.
    // Also save registers to be safe (C++ didn't, but Rust might want to).
    // Actually, saving usage of volatile registers in Rust callback is safer.
    // Let's use asmb::SAVE_REGISTERS
    let mut char_prelude = Vec::new();
    char_prelude.extend_from_slice(&asmb::SAVE_REGISTERS);
    // Execute instruction to load RCX
    char_prelude.extend_from_slice(&[0x48, 0x8B, 0x8B, 0x20, 0x03, 0x00, 0x00]); 
    
    // Check if on_char_update needs to modify RDX/R8... probably not.
    create_hook(
        module_base + offsets::CHAR_UPDATE_HOOK,
        &char_update_old_code,
        &char_prelude, 
        game_state::on_char_update as *const c_void,
        &asmb::RESTORE_REGISTERS 
    );

    // Building Update Hook
    let building_update_old_code: Vec<u8> = vec![
        0x48, 0x8B, 0x43, 0x60,              // mov rax,[rbx+60]
        0x4C, 0x8B, 0x24, 0x28,               // mov r12,[rax+rbp]
        0x49, 0x8B, 0xCC,                     // mov rcx,r12
        0x49, 0x8B, 0x04, 0x24,               // mov rax,[r12]
        0xFF, 0x90, 0xD8, 0x00, 0x00, 0x00  // call qword ptr [rax+000000D8]
    ];
    // We need RCX to be set to r12, which comes from rax+rbp...
    // So we need to execute the full chain to set up RCX.
    let mut build_prelude = Vec::new();
    build_prelude.extend_from_slice(&asmb::SAVE_REGISTERS);
    build_prelude.extend_from_slice(&[0x48, 0x8B, 0x43, 0x60]); // mov rax,[rbx+60]
    build_prelude.extend_from_slice(&[0x4C, 0x8B, 0x24, 0x28]); // mov r12,[rax+rbp]
    build_prelude.extend_from_slice(&[0x49, 0x8B, 0xCC]);       // mov rcx,r12

    create_hook(
        module_base + offsets::BUILDING_UPDATE_HOOK,
        &building_update_old_code,
        &build_prelude,
        game_state::on_building_update as *const c_void,
        &asmb::RESTORE_REGISTERS
    );

    // Spawn Squad Bypass
    let spawn_squad_old_code: Vec<u8> = vec![
        0x48, 0x8D, 0xAC, 0x24, 0x30, 0xFF, 0xFF, 0xFF, // lea rbp,[rsp - 000000D0]
        0x48, 0x81, 0xEC, 0xD0, 0x01, 0x00, 0x00        // sub rsp,000001D0
    ];
    // C++ passes {0x52} (push rdx).
    // The callback bypassSquadSpawningCheck takes `structs::activePlatoon* actvPlatoon`. 
    // This implies arg1 (RCX) is the platoon.
    // If the hook is at `bypass`, maybe RCX is already set?
    // C++ hook doesn't add prelude instructions, just push RDX.
    // Probably safe. We replicate C++ exactly but add register saving if desired.
    // Since we call Rust callback, saving registers is good practice.
    // But `push rdx` suggests `rdx` specifically needs saving? Or maybe `rdx` is the argument?
    // No, standard convention is RCX.
    // Let's stick to C++ simple style for this one + SAVE_REGISTERS.
    let mut spawn_prelude = Vec::new();
    spawn_prelude.extend_from_slice(&asmb::SAVE_REGISTERS);
    // spawn_prelude.push(0x52); // Included in SAVE_REGISTERS
    
    // Note: If C++ pushed RDX, maybe it was to Align Stack? Or used as arg?
    // If we SAVE_ALL, we definitely save RDX.
    
    create_hook(
        module_base + offsets::SPAWN_SQUAD_BYPASS,
        &spawn_squad_old_code,
        &spawn_prelude, 
        game_state::bypass_squad_spawning_check as *const c_void, 
        &asmb::RESTORE_REGISTERS
    );

    // Spawn Squad Injection
    let spawn_squad_injection_old_code: Vec<u8> = vec![
        // 15 NOPs
        0x90, 0x90, 0x90, 0x90,
        0x90, 0x90, 0x90, 0x90,
        0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90
    ];
    // This one returns a value! `void* __fastcall spawnSquadInjection`.
    // And it modifies registers (RAX?).
    // The C++ `aftMyFunc` has complex logic:
    // `pop r10, pop rdx` ...
    // `mov rcx,[rax+38]` ...
    // `cmp rax, 00` ...
    // If `spawnSquadInjection` returns a value in RAX, then `SAVE_REGISTERS` / `RESTORE_REGISTERS` wraps will CLOBBER RAX (restore old RAX).
    // This is valid concern.
    // For this specific hook, we should NOT use the generic SAVE/RESTORE if the return value (RAX) matters.
    // C++ passes: `{0x52, 0x41, 0x52}` (Push RDX, Push R10).
    // And pops them in `aft`.
    // It assumes volatile registers are NOT preserved? Or standard calling convention applies.
    // We should implement it exactly as C++.
    
    let before_inj = vec![0x52, 0x41, 0x52]; 

    let after_inj: Vec<u8> = vec![
        0x41, 0x5A, 0x5A, // pop r10, pop rdx
        // Logic checks RAX (result of callback?).
        // `spawn_squad_injection` returns `void*` (RAX).
        0x48, 0x8B, 0x48, 0x18, // mov rcx,[rax+18]
        0x48, 0x8B, 0x09,       // mov rcx,[rcx]
        0x48, 0x83, 0xF8, 0x00, // cmp rax, 00
        0x0F, 0x84, 0x15, 0x00, 0x00, 0x00, // je ...
        0x4C, 0x8B, 0x08,       // mov r9,[rax]
        0x4C, 0x89, 0x4C, 0x24, 0x30, // mov[rsp + 30],r9
        0x4C, 0x8D, 0x40, 0x08,       // lea r8,[rax + 08]
        0x4C, 0x8B, 0x4E, 0x30,       // mov r9,[rsi + 30]
        0xE9, 0x08, 0x00, 0x00, 0x00, // jmp ...
        0x4C, 0x8B, 0x4E, 0x30, // mov r9,[rsi + 30]
        0x4C, 0x8D, 0x45, 0xA0  // lea r8,[rbp - 60]
    ];
    
    create_hook(
        module_base + offsets::SPAWN_SQUAD_FUNC_CALL,
        &spawn_squad_injection_old_code,
        &before_inj,
        game_state::spawn_squad_injection as *const c_void,
        &after_inj
    );
}
