use std::path::PathBuf;

const FILE: &str = "sonora.log";

pub fn log_file() -> Option<PathBuf> {
    let root = dirs::state_dir().or_else(dirs::cache_dir)?;
    Some(root.join("sonora").join(FILE))
}