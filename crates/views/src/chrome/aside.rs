use gpui::prelude::*;
use gpui::{Context, Render, Window, div};

pub(crate) struct Aside;

impl Aside {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for Aside {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().child("Content")
    }
}