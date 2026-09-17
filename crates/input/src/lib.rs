use gpui::{KeyBinding, actions};
use ui::{
    Activate, Backspace, BackspaceToStart, BackspaceWord, Copy, Cut, Delete, DeleteToEnd,
    DeleteWord, Deselect, Dismiss, End, FORM_CONTEXT, Home, INPUT_CONTEXT, Left, MENU_CONTEXT,
    Paste, Remove, Right, SelectAll, SelectEnd, SelectHome, SelectLeft, SelectNext, SelectPrevious,
    SelectRight, SelectWordLeft, SelectWordRight, ShowCharacterPalette, Space, Submit,
    TABLE_CONTEXT, WordLeft, WordRight,
};

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