use gpui::prelude::*;
use gpui::{Context, Render, Window, div};
use ui::ActiveTheme as _;

pub(crate) struct ToastStack;

impl ToastStack {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ToastStack {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}