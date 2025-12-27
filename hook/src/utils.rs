use windows::Win32::System::Console::{AllocConsole, GetStdHandle, STD_OUTPUT_HANDLE, SetConsoleTitleW};
use windows::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY, PAGE_EXECUTE_READ, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, VirtualProtect, VirtualAlloc, MEM_RESERVE};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use std::ffi::c_void;
use std::ptr;

// ... (previous functions: spawn_console, is_valid_name, scan_memory_for_value)
pub unsafe fn spawn_console() {
    let _ = AllocConsole();
    let title = "Multiplayer Mod Console\0".encode_utf16().collect::<Vec<u16>>();
    let _ = SetConsoleTitleW(windows::core::PCWSTR(title.as_ptr()));
}

pub fn is_valid_name(name: &[u8]) -> bool {
    // ... same as before
    if name.is_empty() { return false; }
    let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    if len < 2 { return false; }
    for &c in &name[..len] {
        if !c.is_ascii_alphanumeric() && c != b' ' && c != b'(' && c != b')' && c != b'_' {
            return false;
        }
    }
    true
}

pub unsafe fn scan_memory_for_value(target_value: u64) -> Vec<usize> {
    // ... same as before
    let mut results = Vec::new();
    let mut sys_info = SYSTEM_INFO::default();
    GetSystemInfo(&mut sys_info);
    let mut start_addr = sys_info.lpMinimumApplicationAddress as usize;
    let end_addr = sys_info.lpMaximumApplicationAddress as usize;
    let mut mem_info = MEMORY_BASIC_INFORMATION::default();
    while start_addr < end_addr {
        if VirtualQuery(Some(start_addr as *const c_void), &mut mem_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) > 0 {
            if mem_info.State == MEM_COMMIT 
               && (mem_info.Protect & (PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY)).0 != 0
               && (mem_info.Protect & PAGE_GUARD).0 == 0
            {
                let region_start = mem_info.BaseAddress as usize;
                let region_size = mem_info.RegionSize;
                let region_end = region_start + region_size;
                let mut aligned_start = (region_start + 7) & !7;
                if aligned_start >= region_end { start_addr = region_end; continue; }
                let aligned_end = region_end - 8;
                if aligned_end < aligned_start { start_addr += region_size; continue; }
                let count = (aligned_end - aligned_start) / 8 + 1;
                let ptr = aligned_start as *const u64;
                for i in 0..count {
                    if *ptr.add(i) == target_value { results.push(aligned_start + i * 8); }
                }
            }
            start_addr += mem_info.RegionSize;
        } else { break; }
    }
    results
}

// Hooking Utils
pub mod asmb {
    // x64 Instructions
    pub const JMP: [u8; 6] = [0xFF, 0x25, 0x00, 0x00, 0x00, 0x00]; // JMP [RIP+0] (absolute jump via table)
    // Actually the C++ code uses FF 25 00 00 00 00 which expects the address 0 bytes after the instruction? 
    // RIP relative addressing. 
    // FF 25 00 00 00 00 -> JMP [RIP+0] -> reads address from the 6 bytes + 0.
    // So address stored at offsets 6..14 (8 bytes).
    
    pub const CALL: [u8; 8] = [0xFF, 0x15, 0x02, 0x00, 0x00, 0x00, 0xEB, 0x08]; 
    pub const NOP: [u8; 1] = [0x90];
    
    pub const SAVE_REGISTERS: [u8; 23] = [
        0x50, 0x51, 0x52, 0x53, 0x55, 0x56, 0x57, 0x41, 0x50,
        0x41, 0x51, 0x41, 0x52, 0x41, 0x53, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57
    ];
    // Push RAX, RCX, RDX, RBX, RBP, RSI, RDI, R8..R15
    // Note: Rust ordering might need verification but this matches C++ vector.
    
    pub const RESTORE_REGISTERS: [u8; 23] = [
        0x41, 0x5F, 0x41, 0x5E, 0x41, 0x5D, 0x41, 0x5C, 0x41,
        0x5B, 0x41, 0x5A, 0x41, 0x59, 0x41, 0x58, 0x5F, 0x5E, 0x5D, 0x5B, 0x5A, 0x59, 0x58
    ];
}

pub unsafe fn create_hook(
    hook_location: usize,
    old_code: &[u8],
    bef_my_func: &[u8], // ignored mostly in C++ usage except specific cases
    my_func: *const c_void,
    aft_my_func: &[u8]
) {
    let min_size = asmb::JMP.len() + 8; // 6 + 8 = 14
    if old_code.len() < min_size {
        println!("create_hook: old_code size too small");
        return;
    }

    let return_location = hook_location + old_code.len();
    
    // Alloc trampoline
    // Size = bef + call(8) + ptr(8) + aft + old + jmp(6) + ptr(8)
    let size = bef_my_func.len() + asmb::CALL.len() + 8 + aft_my_func.len() + old_code.len() + asmb::JMP.len() + 8;
    
    let trampoline = VirtualAlloc(None, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as *mut u8;
    if trampoline.is_null() { return; }
    
    let mut offset = 0;
    
    // Copy SAVE_REGISTERS provided in bef_my_func if any?
    // C++ passes saveRegisters as befMyFunc.
    ptr::copy_nonoverlapping(bef_my_func.as_ptr(), trampoline.add(offset), bef_my_func.len());
    offset += bef_my_func.len();
    
    // Write Call
    ptr::copy_nonoverlapping(asmb::CALL.as_ptr(), trampoline.add(offset), asmb::CALL.len());
    offset += asmb::CALL.len();
    
    // Function Pointer
    ptr::copy_nonoverlapping(&my_func as *const _ as *const u8, trampoline.add(offset), 8);
    offset += 8;
    
    // Copy RESTORE_REGISTERS provided in aft_my_func
    ptr::copy_nonoverlapping(aft_my_func.as_ptr(), trampoline.add(offset), aft_my_func.len());
    offset += aft_my_func.len();
    
    // Copy Old Code
    ptr::copy_nonoverlapping(old_code.as_ptr(), trampoline.add(offset), old_code.len());
    offset += old_code.len();
    
    // JMP Back
    ptr::copy_nonoverlapping(asmb::JMP.as_ptr(), trampoline.add(offset), asmb::JMP.len());
    offset += asmb::JMP.len();
    
    // Return Address
    ptr::copy_nonoverlapping(&return_location as *const _ as *const u8, trampoline.add(offset), 8);
    offset += 8;
    
    // Hook the original location
    let mut old_protect = windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(hook_location as *const c_void, old_code.len(), PAGE_EXECUTE_READWRITE, &mut old_protect);
    
    // Write JMP to Trampoline
    ptr::copy_nonoverlapping(asmb::JMP.as_ptr(), hook_location as *mut u8, asmb::JMP.len());
    // Write Trampoline Address
    let tramp_addr = trampoline as usize;
    ptr::copy_nonoverlapping(&tramp_addr as *const _ as *const u8, (hook_location as *mut u8).add(asmb::JMP.len()), 8);
    
    // NOP remaining bytes
    if old_code.len() > min_size {
        ptr::write_bytes((hook_location as *mut u8).add(min_size), 0x90, old_code.len() - min_size);
    }
    
    VirtualProtect(hook_location as *const c_void, old_code.len(), old_protect, &mut old_protect);
}
