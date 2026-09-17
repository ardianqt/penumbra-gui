pub(crate) fn effects() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("SONORA_BLUR").as_deref() != Ok("0"))
}