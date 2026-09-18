use gpui::{KeyBinding, actions};
use ui::Dismiss;

actions!(
    toolkit,
    [
        Quit,
        NavigateBack,
        NavigateForward,
        OpenSettings,
        CloseWindow,
        MinimizeWindow,
        ZoomWindow,
        ToggleWindowFullscreen,
    ]
);

pub const WORKSPACE_CONTEXT: &str = "Workspace";

pub fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("escape", Dismiss, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
    ]
}