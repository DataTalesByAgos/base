use crate::game_state;
use crate::func;
use std::io::{self, Write};
use std::sync::atomic::Ordering;

pub fn commands_loop() {
    // init() is done in lib.rs
    println!("Console Ready");
    
    let mut buffer = String::new();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        buffer.clear();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let input = buffer.trim();
                if input.is_empty() { continue; }
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.is_empty() { continue; }
                
                let command = parts[0];
                let args = &parts[1..];
                
                match command {
                    "help" => help(args),
                    "chars" => chars(args),
                    "builds" => builds(args),
                    "clear" => clear(args),
                    "give" => spawn_item(args),
                    "db" => search_db(args),
                    "heapScan" => heap_scan(args),
                    // "call" => call(args), // Unsafe dynamic call, maybe skip or implement later
                    _ => println!("Unknown command: \"{}\". Type \"help\" to see a list of commands.", command),
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }
    }
}

fn help(_args: &[&str]) {
    println!("List of Commands:");
    println!("help");
    println!("chars [name]");
    println!("builds [name]");
    println!("clear");
    println!("give <char*> <data*> [sectionString]");
    println!("db <name>");
    println!("heapScan");
}

fn chars(args: &[&str]) {
    unsafe {
        if let Some(ref chars) = game_state::CHARS {
            if chars.is_empty() {
                println!("No characters found");
                return;
            }
            println!("List of characters:");
            for (addr, entry) in chars.iter() {
                if args.is_empty() || entry.name.starts_with(args[0]) {
                     let now = windows::Win32::System::SystemInformation::GetTickCount64();
                     let since = (now - entry.last_seen) as f64 / 1000.0;
                     println!("Addr: {:X} Name: {}, Last seen: {}s", addr, entry.name, since);
                }
            }
        } else {
             println!("CHARS not initialized");
        }
    }
}

fn builds(args: &[&str]) {
    unsafe {
        if let Some(ref builds) = game_state::BUILDS {
             if builds.is_empty() {
                println!("No buildings found");
                return;
            }
            println!("List of buildings:");
            for (addr, entry) in builds.iter() {
                if args.is_empty() || entry.name.starts_with(args[0]) {
                     let now = windows::Win32::System::SystemInformation::GetTickCount64();
                     let since = (now - entry.last_seen) as f64 / 1000.0;
                     println!("Addr: {:X} Name: {}, Last seen: {}s", addr, entry.name, since);
                }
            }
        }
    }
}

fn clear(_args: &[&str]) {
    // Windows CLS?
    // std::process::Command::new("cls").status().unwrap(); // Might not work in attached console
    print!("\x1B[2J\x1B[1;1H"); // ANSI escape codes if terminal supports it
}

fn spawn_item(args: &[&str]) {
    if args.is_empty() || args[0] == "info" {
         println!("give <char*> <data*> [sectionString]");
         return;
    }
    if args.len() < 2 { return; }
    
    // Parse hex addresses
    let char_addr = usize::from_str_radix(args[0].trim_start_matches("0x"), 16).unwrap_or(0);
    let item_data_addr = usize::from_str_radix(args[1].trim_start_matches("0x"), 16).unwrap_or(0);
    
    if char_addr == 0 || item_data_addr == 0 {
        println!("Invalid address");
        return;
    }
    
    // unsafe call to logic using func.rs
    // TODO: implement logic similar to commands.cpp spawnItem
    // requires wrapping func::spawn_item and func::get_inv_section
}

fn search_db(args: &[&str]) {
    unsafe {
        if let Some(ref db) = game_state::DB {
            if db.is_empty() { println!("No entries found"); return; }
            let query = args.join(" ");
            for (name, addr) in db.iter() {
                if query.is_empty() || name.starts_with(&query) {
                    println!("Name: {}, Adr: {:X}", name, addr);
                }
            }
        }
    }
}

fn heap_scan(_args: &[&str]) {
    println!("Scanning heap...");
    unsafe { game_state::scan_heap(); }
    println!("Scan complete.");
}
