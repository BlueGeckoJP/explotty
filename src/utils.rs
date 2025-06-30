// Unix-like systems only
pub fn get_current_dir_from_pty(pid: u32) -> Option<String> {
    #[cfg(unix)]
    {
        let cwd_path = format!("/proc/{pid}/cwd");
        match std::fs::read_link(cwd_path) {
            Ok(path) => Some(path.to_string_lossy().into_owned()),
            Err(_) => None,
        }
    }

    #[cfg(not(unix))]
    {
        warn!("get_current_dir_from_pty is only implemented for Unix-like systems");
        None
    }
}
