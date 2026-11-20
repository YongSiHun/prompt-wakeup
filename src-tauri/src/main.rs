#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 진입: 앱·실행 래퍼 분기.
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => prompt_wakeup::run(),
        [flag, id] if flag == "--run-job" => run_job(id),
        _ => {
            eprintln!("지원하지 않는 실행 인자");
            std::process::exit(2);
        }
    }
}
// 실행: 예약 작업 래퍼.
fn run_job(id: &str) {
    if let Err(error) = prompt_wakeup::execution::run_job(id) {
        eprintln!("PromptWakeup: {error}");
        std::process::exit(1);
    }
}
