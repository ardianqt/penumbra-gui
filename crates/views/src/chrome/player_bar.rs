use gpui::prelude::*;
use gpui::{App, Context, Render, Window, div, px};
use ui::ActiveTheme as _;

pub(crate) struct PlayerBar;

impl PlayerBar {
    pub fn new() -> Self {
        Self
    }

    pub fn height(_window: &Window, _cx: &App) -> gpui::Pixels {
        gpui::px(48.)
    }
}

impl Render for PlayerBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .px_4()
            .h(px(48.))
            .border_t_1()
            .border_color(cx.theme().sidebar_border)
            .bg(cx.theme().sidebar)
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .text_sm()
                    .child("Toolkit v1.0"),
            )
    }
}