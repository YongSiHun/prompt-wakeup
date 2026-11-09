#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => prompt_wakeup::run(),
        [flag, id] if flag == "--run-job" => run_job(id),
        _ => { eprintln!("지원하지 않는 실행 인자"); std::process::exit(2); }
    }
}
fn run_job(id: &str) {
    attach_console();
    if let Err(error) = prompt_wakeup::execution::run_job(id) {
        eprintln!("PromptWakeup: {error}");
        std::process::exit(1);
    }
}
#[cfg(windows)]
fn attach_console() {
    // The visible terminal is the parent; this never allocates an internal terminal.
    unsafe { windows_sys::Win32::System::Console::AttachConsole(windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS); }
}
#[cfg(not(windows))]
fn attach_console() {}
