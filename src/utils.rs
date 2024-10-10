use crate::{cli::Args, MemoryMode};

pub fn get_memory_mode(args: &Args, file_size: u64) -> MemoryMode {
    const THRESHOLD: u64 = 1000 * 1000 * 500; // 500MB

    // Disallow simulataneously using disable_lmm and lmm flags
    if args.low_memory_mode && args.disable_low_memory_mode {
        log::error!("You cannot have both --low-memory-mode and --disable-low-memory-mode flag active at the same time!");
        panic!("Incompatible flags active together.");
    }

    let mut is_low_memory_mode_enabled = args.low_memory_mode && !args.disable_low_memory_mode;

    // Enable for files 500MB+
    if file_size > THRESHOLD && !args.disable_low_memory_mode {
        is_low_memory_mode_enabled = true;
    }

    if is_low_memory_mode_enabled {
        MemoryMode::Low
    } else {
        MemoryMode::Standard
    }
}
