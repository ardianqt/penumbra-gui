use gpui::prelude::*;
use gpui::{Context, Render, Window, div};
use ui::ActiveTheme as _;

pub(crate) struct UnisocView;

impl Render for UnisocView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_8()
            .child(
                div()
                    .text_2xl()
                    .child("Unisoc"),
            )
            .child(
                div()
                    .mt_4()
                    .text_color(cx.theme().muted_foreground)
                    .child("Toolkit page - functionality coming soon"),
            )
    }
}