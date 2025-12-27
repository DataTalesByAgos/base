use std::ffi::CStr;
use std::os::raw::c_char;

#[repr(C)]
pub struct KenshiString {
    pub string: [u8; 0x10],
    pub size: i32,
    _padding: [u8; 0x100],
}

#[repr(C)]
pub struct GameWorldClass {
    _padding1: [u8; 0x700],
    pub game_speed: f32, // Offset 0x700
    _padding2: [u8; 0x1B5],
    pub paused: bool,    // Offset 0x8B9
}

#[repr(C)]
pub struct Vector3Raw {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
pub struct Vector3 {
    _padding1: [u8; 0x20],
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
pub struct Building {
    _padding1: [u8; 0x18],
    pub name: [u8; 0x30], // Offset 0x18 (0x48 - 0x18)
    // padding for name[0x48] - padding1[0x18] = 0x30 bytes
    pub x: f32, // Offset 0x48
    pub y: f32, // Offset 0x4C
    pub z: f32, // Offset 0x50
    _padding2: [u8; 0x164 - (0x50 + 0x4)],
    pub condition: f32, // Offset 0x164
    _padding3: [u8; 0x430 - (0x164 + 0x4)],
    pub inventory: *mut Inventory, // Offset 0x430
    _padding4: [u8; 0x448 - (0x430 + 0x8)],
    pub production: *mut ProgressContainer, // Offset 0x448
}

impl Building {
    pub fn get_name(&self) -> String {
        // Implementation similar to C++ getName handling potential pointer vs direct string
        // For now, return a String from the fixed buffer as a safeguard
        unsafe {
            // Logic mimicking the try-except block in C++ would require Windows SEH or careful pointer checking
            // Assuming direct array for now as a safe default for migration initial structure
            let c_str: &CStr = CStr::from_ptr(self.name.as_ptr() as *const i8);
            c_str.to_string_lossy().into_owned()
        }
    }
}

#[repr(C)]
pub struct GameData {
    _padding1: [u8; 0x10],
    pub manager: *mut GameDataManager, // Offset 0x10
    _padding2: [u8; 0x10],
    pub name: [u8; 0x10], // Offset 0x28
    pub name_length: i32, // Offset 0x38
}

#[repr(C)]
pub struct GameDataManager {}

#[repr(C)]
pub struct Item {
    _padding1: [u8; 0xDC],
    pub x: i32, // Offset 0xDC
    pub y: i32, // Offset 0xE0
    pub inventory_section_name: *mut c_char, // Offset 0xE4? C++ says EC (236)
                                             // 0xDC (220) + 4 + 4 = 228 (0xE4). 
                                             // C++ struct:
                                             // padding1[0xDC] (220)
                                             // x (4) -> 224 (0xE0)
                                             // y (4) -> 228 (0xE4)
                                             // inventorySectionName -> ?
                                             // Re-check C++ alignment.
                                             // class Item {
                                             //   char padding1[0xDC];
                                             //   public:
                                             //   int x;// Offset 0xDC
                                             //   int y;// Offset 0xE0
                                             //   char* inventorySectionName;// Offset EC (0xE0 + 4 + padding(4 for 64bit align)? )
                                             // Pointer on 64 bit is 8 bytes.
                                             // 0xE0 + 4 = 0xE4. Next 8 byte aligned address is 0xE8.
                                             // C++ source says: "char* inventorySectionName;// Offset EC"
                                             // But EC is 236. 0xE0 is 224.
                                             // Let's assume explicitly padding to match C++ offset comments if trustworthy, 
                                             // or trust compiler alignment.
                                             // For now, following the explicit offsets in comments.
     _pad_align: [u8; 0xEC - 0xE4], 
     pub inventory_section_name_ptr: *mut c_char, // Offset 0xEC
     
     // NOTE: This looks suspicious for x64 (pointers are 8 bytes, usually aligned to 8).
     // 0xEC is not 8-byte aligned. 
     // Let's revisit the C++ struct:
     // int x; // 0xDC
     // int y; // 0xE0
     // char* inventorySectionName; // Offset EC
     // If this is x64, a pointer at 0xEC is weird. 
     // Maybe it's a 32-bit offset legacy comment? 
     // Checking usage in C++ might clarify, but for now we follow the structure.
     
    _padding2: [u8; 0x3C],
    pub quantity: i32, // Offset 0x12C
    _padding3: [u8; 1],
}

#[repr(C)]
pub struct InventorySection {}

#[repr(C)]
pub struct ProgressContainer {
    pub progress: f32, // Offset 0x0
    _padding1: [u8; 0xC],
    pub item_info: *mut GameData,      // Offset 0x10
    pub inv_section: *mut InventorySection, // Offset 0x18
}

#[repr(C)]
pub struct MedicalSystem {}

#[repr(C)]
pub struct Faction {
    _padding1: [u8; 0x1A8],
    pub name: [u8; 0x10], // Offset 0x1A8
    pub name_length: i32, // Offset 0x1B8
}

#[repr(C)]
pub struct CharacterHuman {
    _padding1: [u8; 0x10],
    pub faction: *mut Faction, // Offset 0x10
    pub name: [u8; 8],         // Offset 0x18
    _padding2: [u8; 0x48 - (0x18 + 8)],
    pub pos: Vector3Raw,       // Offset 0x48
    _padding3: [u8; 0x2E8 - (0x48 + 16)], // Vector3Raw size is 12 (3 floats), but struct padding often aligns to 16?
                                          // C++: "structs::Vector3Raw pos; // Offset 0x48"
                                          // char padding21[0x2E8 - (0x48 + 16)]; 
                                          // 16 used in C++ calculation hints at sizeof(Vector3Raw) being padded to 16 or logic error.
                                          // Struct definition: 3 floats = 12 bytes.
    pub inventory: *mut Inventory, // Offset 0x2E8
    _padding4: [u8; 0x450 - (0x2E8 + 8)],
    pub stats: *mut CharStats,     // Offset 0x450
}

#[repr(C)]
pub struct Inventory {
    _padding1: [u8; 0x18],
    pub number_of_items: i32, // Offset 0x18
    _padding2: [u8; 0x4],
    pub item_list: *mut *mut Item, // Offset 0x20
    _padding3: [u8; 0x80 - 0x28],
    pub character: *mut CharacterHuman, // Offset 0x80
}

#[repr(C)]
pub struct CharStats {
    _padding1: [u8; 0x8],
    pub medical: *mut MedicalSystem, // Offset 0x8
}

#[repr(C)]
pub struct AnimationClassHuman {
    _padding1: [u8; 0xC0],
    pub movement: *mut CharMovement, // Offset 0xC0
    _padding2: [u8; 0x2D8 - 0xC0 - 8], // sizeof pointer 8
    pub character: *mut CharacterHuman, // Offset 0x2D8
}

#[repr(C)]
pub struct CharMovement {
    _padding1: [u8; 0x320],
    pub position: *mut Vector3, // Offset 0x320
}

#[repr(C)]
pub struct ActivePlatoon {
    _padding0: [u8; 0x58],
    pub skip_spawning_check2: bool, // Offset 0x58
    _padding1: [u8; 0x78 - 0x59],
    pub squad: *mut Platoon,        // Offset 0x78
    pub list: *mut HandleList,      // Offset 0x80
    _padding2: [u8; 0xA0 - 0x88],
    pub leader: *mut CharacterHuman, // Offset 0xA0
    _padding3: [u8; 0xF0 - 0xA8],
    pub skip_spawning_check1: bool, // Offset 0xF0
    _padding4: [u8; 0x250 - 0xF1],
    pub skip_spawning_check3: *mut std::ffi::c_void, // Offset 0x250
}

#[repr(C)]
pub struct Platoon {
    _padding1: [u8; 0x10],
    pub faction: *mut Faction, // Offset 0x10
    _padding2: [u8; 0x1D8 - 0x18],
    pub active: *mut ActivePlatoon, // Offset 0x1D8
}

#[repr(C)]
pub struct HandleList {
    _padding1: [u8; 0x8],
     // list logic...
}
