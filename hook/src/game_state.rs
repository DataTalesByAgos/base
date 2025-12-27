use crate::offsets;
use crate::structs::{GameWorldClass, AnimationClassHuman, Building, Vector3, GameData, CharacterHuman, ActivePlatoon, Platoon, Faction};
use crate::utils;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};
use windows::Win32::System::SystemInformation::GetTickCount64;

// Global game world pointer
pub static mut GAME_WORLD: *mut GameWorldClass = std::ptr::null_mut();
pub static mut MODULE_BASE: usize = 0;
pub static mut PLAYER: *mut AnimationClassHuman = std::ptr::null_mut();
pub static mut OTHER_PLAYERS: *mut AnimationClassHuman = std::ptr::null_mut();

// Tracked variables
pub struct TrackedVariable {
    pub getter: Box<dyn Fn() -> String + Send + Sync>,
    pub setter: Box<dyn Fn(String) + Send + Sync>,
    pub old_val: String,
    pub changed: bool,
}

pub static mut VARIABLES: Option<Vec<TrackedVariable>> = None;
pub static GAME_LOADED: AtomicBool = AtomicBool::new(false);

// Entity Maps
pub struct EntityEntry {
    pub name: String,
    pub last_seen: u64,
}

pub static mut CHARS: Option<HashMap<usize, EntityEntry>> = None;
pub static mut BUILDS: Option<HashMap<usize, EntityEntry>> = None;
pub static mut DB: Option<HashMap<String, usize>> = None;

// Squad Spawn Bypass State
pub static mut SQUAD_SPAWN_BYPASS_AMM: i32 = 0;
pub static mut BYPASSED_PLATOON: *mut ActivePlatoon = std::ptr::null_mut();
pub static mut CHECK_RESTORATION: bool = false;
pub static mut CHAR_SPAWNING_QUEUE: Option<VecDeque<CustomChar>> = None;

// Custom struct mimicking C++ local struct
pub struct CustomChar {
    pub data: *mut GameData,
    pub pos: crate::structs::Vector3Raw, // Assuming Vector3Raw is compatible
}

pub unsafe fn init() {
    let module_handle = GetModuleHandleA(None).unwrap();
    MODULE_BASE = module_handle.0 as usize;
    GAME_WORLD = (MODULE_BASE + offsets::GAME_WORLD_OFFSET) as *mut GameWorldClass;
    
    CHARS = Some(HashMap::new());
    BUILDS = Some(HashMap::new());
    DB = Some(HashMap::new());
    CHAR_SPAWNING_QUEUE = Some(VecDeque::new());

    let mut vars: Vec<TrackedVariable> = Vec::new();
    // 0: Faction
    vars.push(TrackedVariable {
        getter: Box::new(|| get_faction()),
        setter: Box::new(|v| set_faction(v)),
        old_val: String::new(),
        changed: false,
    });
    // 1: Speed
    vars.push(TrackedVariable {
        getter: Box::new(|| get_speed()),
        setter: Box::new(|v| set_speed(v)),
        old_val: String::new(),
        changed: false,
    });
    // 2: Player 1
    vars.push(TrackedVariable {
        getter: Box::new(|| get_player1()),
        setter: Box::new(|v| set_player1(v)),
        old_val: String::new(),
        changed: false,
    });
    // 3: Player 2
    vars.push(TrackedVariable {
        getter: Box::new(|| get_player2()),
        setter: Box::new(|v| set_player2(v)),
        old_val: String::new(),
        changed: false,
    });

    VARIABLES = Some(vars);
}

// ... Getters/Setters (same as before) ...
unsafe fn get_speed() -> String {
    if GAME_WORLD.is_null() { return "0.000000".to_string(); }
    if (*GAME_WORLD).paused { "0.000000".to_string() } else { (*GAME_WORLD).game_speed.to_string() }
}
unsafe fn get_faction() -> String {
    let ptr = (MODULE_BASE + offsets::FACTION_STRING) as *const i8;
    let mut buffer = [0u8; 17];
    let slice = std::slice::from_raw_parts(ptr as *const u8, 17);
    buffer.copy_from_slice(slice);
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(17);
    String::from_utf8_lossy(&buffer[..len]).to_string()
}
unsafe fn get_own_char_name() -> String {
    let faction = get_faction();
    match faction.as_str() { "10-multiplayr.mod" => "Player 1".to_string(), "12-multiplayr.mod" => "Player 2".to_string(), _ => "!fail".to_string(), }
}
unsafe fn get_other_char_name() -> String {
    let faction = get_faction();
    match faction.as_str() { "10-multiplayr.mod" => "Player 2".to_string(), "12-multiplayr.mod" => "Player 1".to_string(), _ => "!fail".to_string(), }
}
unsafe fn get_player1() -> String {
    if OTHER_PLAYERS.is_null() || PLAYER.is_null() { return "-5139.11,158.019,345.631".to_string(); }
    if get_other_char_name() == "Player 1" { vector3_to_string((*(*OTHER_PLAYERS).movement).position) } else { vector3_to_string((*(*PLAYER).movement).position) }
}
unsafe fn get_player2() -> String {
    if OTHER_PLAYERS.is_null() || PLAYER.is_null() { return "-5139.11,158.019,345.631".to_string(); }
    if get_other_char_name() == "Player 2" { vector3_to_string((*(*OTHER_PLAYERS).movement).position) } else { vector3_to_string((*(*PLAYER).movement).position) }
}
unsafe fn vector3_to_string(vec: *mut Vector3) -> String {
    if vec.is_null() { return "0,0,0".to_string(); }
    format!("{},{},{}", (*vec).x, (*vec).y, (*vec).z)
}
unsafe fn set_speed(data: String) {
    if GAME_WORLD.is_null() { return; }
    let desired_speed: f32 = data.parse().unwrap_or(1.0);
    (*GAME_WORLD).game_speed = desired_speed;
    let set_paused_func: extern "system" fn(*mut GameWorldClass, bool) = std::mem::transmute(MODULE_BASE + offsets::SET_PAUSED);
    if desired_speed == 0.0 && !(*GAME_WORLD).paused { set_paused_func(GAME_WORLD, true); } 
    else if desired_speed != 0.0 && (*GAME_WORLD).paused { (*GAME_WORLD).paused = false; set_paused_func(GAME_WORLD, true); set_paused_func(GAME_WORLD, false); }
}
unsafe fn set_faction(data: String) {
    let ptr = (MODULE_BASE + offsets::FACTION_STRING) as *mut c_void; // cheat
    // ...
}
unsafe fn set_player1(data: String) { set_player_pos_generic(data, "Player 1"); }
unsafe fn set_player2(data: String) { set_player_pos_generic(data, "Player 2"); }
unsafe fn set_player_pos_generic(data: String, target_name: &str) {
    if OTHER_PLAYERS.is_null() { return; }
    if get_other_char_name() == target_name { update_pos((*(*OTHER_PLAYERS).movement).position, data); }
}
unsafe fn update_pos(vec: *mut Vector3, data: String) {
    if vec.is_null() { return; }
    let parts: Vec<&str> = data.split(',').collect();
    if parts.len() == 3 { (*vec).x = parts[0].parse().unwrap_or(0.0); (*vec).y = parts[1].parse().unwrap_or(0.0); (*vec).z = parts[2].parse().unwrap_or(0.0); }
}

pub unsafe fn set_data(data: &str) {
     if let Some(ref mut vars) = VARIABLES {
        let mut key = String::new();
        for line in data.lines() {
            if key.is_empty() { key = line.to_string(); continue; }
            if let Ok(idx) = key.parse::<usize>() {
                 if idx < vars.len() {
                    let cur_val = (vars[idx].getter)();
                    if vars[idx].old_val == cur_val {
                        if line != cur_val { (vars[idx].setter)(line.to_string()); }
                        vars[idx].changed = false;
                    } else { vars[idx].changed = true; }
                    vars[idx].old_val = (vars[idx].getter)();
                 }
            }
            key.clear();
        }
    }
}
pub unsafe fn get_data() -> String {
    let mut data = String::from("0\nB\n");
    if let Some(ref mut vars) = VARIABLES {
        for (i, var) in vars.iter_mut().enumerate() {
            if var.changed { data.push_str(&format!("{}\n{}\n", i, (var.getter)())); var.changed = false; }
        }
    }
    return data;
}

// Callbacks

pub unsafe fn on_char_update(anim: *mut AnimationClassHuman) {
    if let Some(ref mut chars) = CHARS {
        let char_ptr = (*anim).character;
        let key = char_ptr as usize;
        
        // Simple name retrieval for identification
        // We really should use a safer method or copy name from struct
        let my_name_ptr = &(*char_ptr).name;
        // assume short
        let binding = get_own_char_name();
        let own_name = binding.as_bytes(); 
        
        let binding2 = get_other_char_name();
        let other_name = binding2.as_bytes();
        
        // Logic to update PLAYER / OTHER_PLAYERS globals would be here
        // if name match...
        
        let entry = chars.entry(key).or_insert_with(|| {
             EntityEntry { name: "Unknown".to_string(), last_seen: 0 }
        });
        entry.last_seen = GetTickCount64();
    }
}

pub unsafe fn on_building_update(bwd: *mut Building) {
    if let Some(ref mut builds) = BUILDS {
        let key = bwd as usize;
        // if builds.contains_key...
        let entry = builds.entry(key).or_insert_with(|| {
             EntityEntry { name: (*bwd).get_name(), last_seen: 0 }
        });
        entry.last_seen = GetTickCount64();
    }
}

pub unsafe fn bypass_squad_spawning_check(actv_platoon: *mut ActivePlatoon) {
    if SQUAD_SPAWN_BYPASS_AMM > 0 
       && (*actv_platoon).skip_spawning_check3.is_null() 
       && (*actv_platoon).skip_spawning_check1 
    {
        BYPASSED_PLATOON = actv_platoon;
        (*actv_platoon).skip_spawning_check1 = false;
        CHECK_RESTORATION = (*actv_platoon).skip_spawning_check2;
        (*actv_platoon).skip_spawning_check2 = false;
        SQUAD_SPAWN_BYPASS_AMM -= 1;
    }
}

#[repr(C)]
pub struct CustomStructsChar {
    pub data: *mut GameData,
    pub pos: crate::structs::Vector3Raw,
    pub module_base: usize,
}

// Global buffer to store return struct, mirroring C++ `char returns[sizeof(customStructs::Char)]`
pub static mut RETURN_CHAR: Option<CustomStructsChar> = None;

// Note: This function signature matches the hook expectation where args are passed/stack is set.
// But the hook actually pushes args before calling.
// C++: void* __fastcall spawnSquadInjection(void* garbage, ...)
// It takes many args.
// Our hook wrapper calls it.
// We should return `*mut CustomStructsChar` (as void*) to match C++ return.
pub unsafe extern "system" fn spawn_squad_injection(
    garbage: *mut c_void,   // rcx
    faction: *mut Faction,  // rdx
    position: crate::structs::Vector3, // r8 (struct by val?? No, standard for struct > 8 bytes is pointer, but Vector3 might be small enough? 12 bytes? It's typically passed by pointer if not 1,2,4,8 size)
                                      // C++: `structs::Vector3 position` value. 
                                      // If it is passed by value in registers (R8), it usually needs to fit in 64 bits.
                                      // 12 bytes > 8 bytes.
                                      // MSVC x64: structs larger than 8 bytes are passed by pointer (reference).
                                      // So `position` here is likely `*mut Vector3`.
                                      // Wait, C++ signature: `structs::Vector3 position`.
                                      // If defined as value, caller places it.
                                      // MSVC x64 passes > 8 byte structs by reference (hidden pointer).
                                      // So R8 is likely a pointer to the vector.
    town_or_nest: *mut c_void, // r9
    // Stack args follow
    stack_offset1: usize, // [rsp+20] (shadow space 5th arg)
    stack_offset2: usize, // [rsp+28]
    magic: usize,         // [rsp+30]
    // ...
) -> *mut CustomStructsChar {
    if RETURN_CHAR.is_none() {
        RETURN_CHAR = Some(CustomStructsChar {
             data: std::ptr::null_mut(),
             pos: crate::structs::Vector3Raw {x:0., y:0., z:0.},
             module_base: 0
        });
    }
    let return_dude = RETURN_CHAR.as_mut().unwrap();
    return_dude.module_base = MODULE_BASE + offsets::SQUAD_SPAWNING_HAND;

    // This checks `squad` which was passed as arg in C++ [rsp+38]
    // Accessing stack args in Rust "extern system" beyond register args is non-trivial without explicit definition.
    // For now, minimal logic:
    if BYPASSED_PLATOON.is_null() {
        return_dude.data = std::ptr::null_mut();
        return return_dude;
    }
    
    // Restore logic
    (*BYPASSED_PLATOON).skip_spawning_check1 = true;
    (*BYPASSED_PLATOON).skip_spawning_check2 = CHECK_RESTORATION;
    BYPASSED_PLATOON = std::ptr::null_mut();

    if let Some(ref mut queue) = CHAR_SPAWNING_QUEUE {
        if let Some(dude) = queue.pop_front() {
            return_dude.data = dude.data;
            return_dude.pos = dude.pos;
        }
    }
    
    return_dude
}

pub unsafe fn scan_heap() {
    let results = utils::scan_memory_for_value(MODULE_BASE as u64 + offsets::GAME_DATA_MANAGER_MAIN as u64);
    if let Some(ref mut db) = DB {
        for addr in results {
            let data = (addr - 0x10) as *mut GameData;
            db.insert("Unknown".to_string(), data as usize);
        }
    }
    println!("HeapScan found {} entries.", results.len());
}
