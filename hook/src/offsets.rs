pub mod offsets {
    // GOG 1.0.68 - x64
    pub const FACTION_STRING: usize = 0x16C2F68;
    pub const GAME_WORLD_OFFSET: usize = 0x2133040;
    pub const SET_PAUSED: usize = 0x7876A0;
    pub const CHAR_UPDATE_HOOK: usize = 0x65F6C7;
    pub const BUILDING_UPDATE_HOOK: usize = 0x9FAA57;

    pub const ITEM_SPAWNING_HAND: usize = 0x1E395F8;
    pub const ITEM_SPAWNING_MAGIC: usize = 0x21334E0;
    pub const SPAWN_ITEM_FUNC: usize = 0x2E41F;
    pub const GET_SECTION_FROM_INV_BY_NAME: usize = 0x4FE3F;

    pub const GAME_DATA_MANAGER_MAIN: usize = 0x2133060;
    pub const GAME_DATA_MANAGER_FOLIAGE: usize = 0x21331E0;
    pub const GAME_DATA_MANAGER_SQUADS: usize = 0x2133360;

    pub const SPAWN_SQUAD_BYPASS: usize = 0x4FF47C;
    pub const SPAWN_SQUAD_FUNC_CALL: usize = 0x4FFA88;
    pub const SQUAD_SPAWNING_HAND: usize = 0x21334E0;
}
