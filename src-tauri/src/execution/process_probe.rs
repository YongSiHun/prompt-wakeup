/// A missing process is evidence of lost tracking, never evidence of CLI success.
pub fn alive(pid:u32)->bool {
    #[cfg(windows)] unsafe {use windows_sys::Win32::{Foundation::{CloseHandle,GetLastError,ERROR_INVALID_PARAMETER},System::Threading::{OpenProcess,GetExitCodeProcess,PROCESS_QUERY_LIMITED_INFORMATION}};let process=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,0,pid);if process.is_null(){return GetLastError()!=ERROR_INVALID_PARAMETER}let mut code=0;let ok=GetExitCodeProcess(process,&mut code);CloseHandle(process);ok==0||code==259}
    #[cfg(unix)] {std::process::Command::new("/bin/kill").args(["-0",&pid.to_string()]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s|s.success()).unwrap_or(true)}
}
