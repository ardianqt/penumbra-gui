use gpui::{App, Global, Window};
use ui::Rounding;

type Apply = Box<dyn Fn(&Window, Rounding)>;

struct RoundedWindowHook(Apply);

impl Global for RoundedWindowHook {}

pub fn install_rounded_window_hook(apply: impl Fn(&Window, Rounding) + 'static, cx: &mut App) {
    cx.set_global(RoundedWindowHook(Box::new(apply)));
}

pub fn apply_window_rounding(window: &Window, rounding: Rounding, cx: &App) {
    if let Some(hook) = cx.try_global::<RoundedWindowHook>() {
        (hook.0)(window, rounding);
    }
}