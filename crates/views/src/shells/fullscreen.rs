use gpui::prelude::*;
use gpui::{Context, Render, Window, div};

pub(crate) struct FullscreenView;

impl FullscreenView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }

    pub fn focus(&self, _window: &mut Window, _cx: &mut App) {}
}

impl Render for FullscreenView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}