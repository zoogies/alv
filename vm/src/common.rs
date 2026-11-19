// I don't want to have to mentally think about cargo feature flags.

// Debugging Stuff
pub const DEBUG_TRACE_EXECUTION:    bool = cfg!(debug_assertions);
pub const DEBUG_PRINT_CODE:         bool = cfg!(debug_assertions);

// Core VM Constants
pub const FRAMES_MAX: usize = 64;

// Choices for compiling alv
pub const ALV_NOSTD: bool = false;