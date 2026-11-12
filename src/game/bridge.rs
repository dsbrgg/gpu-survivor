use serde::Serialize;
use std::cell::RefCell;

#[derive(Default)]
pub struct Commands {
    pub start: Option<u32>,
    pub pause: bool,
    pub upgrade: Option<usize>,
    pub menu: bool,
    pub live_update: bool,
}

thread_local! {
    static COMMANDS: RefCell<Commands> = RefCell::new(Commands::default());
}

#[no_mangle]
pub extern "C" fn gpu_bridge_crate_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn gpu_start(character: u32) {
    COMMANDS.with(|c| {
        c.borrow_mut().start = Some(character.min(crate::founder::FOUNDERS.len() as u32 - 1))
    });
}

#[no_mangle]
pub extern "C" fn gpu_pause() {
    COMMANDS.with(|c| c.borrow_mut().pause = true);
}

#[no_mangle]
pub extern "C" fn gpu_upgrade(index: u32) {
    COMMANDS.with(|c| c.borrow_mut().upgrade = Some(index as usize));
}

#[no_mangle]
pub extern "C" fn gpu_menu() {
    COMMANDS.with(|c| c.borrow_mut().menu = true);
}

#[no_mangle]
pub extern "C" fn gpu_live_update() {
    COMMANDS.with(|c| c.borrow_mut().live_update = true);
}

pub fn take_commands() -> Commands {
    COMMANDS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn gpu_config_len() -> u32;
    fn gpu_config_read(pointer: *mut u8, length: u32);
    fn gpu_live_len() -> u32;
    fn gpu_live_read(pointer: *mut u8, length: u32);
    fn gpu_event(pointer: *const u8, length: u32);
}

pub fn live_configuration() -> String {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let length = gpu_live_len().min(32_768);
        let mut bytes = vec![0; length as usize];
        gpu_live_read(bytes.as_mut_ptr(), length);
        String::from_utf8(bytes).unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

pub fn configuration() -> String {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let length = gpu_config_len().min(32_768);
        let mut bytes = vec![0; length as usize];
        gpu_config_read(bytes.as_mut_ptr(), length);
        String::from_utf8(bytes).unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    serde_json::to_string(&super::config::Edition::demo()).unwrap()
}

pub fn emit(value: &impl Serialize) {
    #[cfg(target_arch = "wasm32")]
    if let Ok(json) = serde_json::to_string(value) {
        unsafe { gpu_event(json.as_ptr(), json.len() as u32) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = value;
}
