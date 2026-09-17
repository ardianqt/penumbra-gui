use gpui::prelude::*;
use gpui::{Context, Render, Window, div};

pub(crate) struct UpdateNotice;

impl UpdateNotice {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for UpdateNotice {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}