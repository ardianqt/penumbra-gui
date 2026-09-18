use gpui::prelude::*;
use gpui::{AnyView, Context, EventEmitter, Render, Window, div, px};
use router::{back, forward};
use ui::{ActiveTheme as _, Button};

pub(crate) struct TitleBarOptions {
    pub navigation: bool,
    pub sidebar_open: bool,
    pub sidebar_right: Option<bool>,
    pub offset: gpui::Pixels,
    pub border: bool,
    pub content: Option<AnyView>,
}

impl Default for TitleBarOptions {
    fn default() -> Self {
        Self {
            navigation: true,
            sidebar_open: true,
            sidebar_right: None,
            offset: gpui::Pixels::ZERO,
            border: true,
            content: None,
        }
    }
}

pub(crate) enum TitleBarEvent {
    ToggleSidebar,
    ToggleSidebarRight,
}

pub(crate) struct TitleBar {
    options: TitleBarOptions,
}

impl EventEmitter<TitleBarEvent> for TitleBar {}

impl TitleBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            options: TitleBarOptions::default(),
        }
    }

    pub fn set_options(&mut self, options: TitleBarOptions, cx: &mut Context<Self>) {
        self.options = options;
        cx.notify();
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .flex()
            .items_center()
            .h(px(38.))
            .w_full()
            .flex_none()
            .px_2()
            .gap_1()
            .when(self.options.border, |this| this.border_b_1())
            .border_color(theme.sidebar_border)
            .bg(theme.sidebar)
            .child(
                Button::new("sidebar-toggle")
                    .icon("icons/menu.svg")
                    .ghost()
                    .small()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(TitleBarEvent::ToggleSidebar);
                    })),
            )
            .when(self.options.navigation, |this| {
                this.child(
                    div().flex().items_center().gap_px().px_2().child(
                        Button::new("history-back")
                            .icon("icons/chevron-left.svg")
                            .ghost()
                            .small()
                            .on_click(|_, _, cx| back(cx)),
                    )
                    .child(
                        Button::new("history-forward")
                            .icon("icons/chevron-right.svg")
                            .ghost()
                            .small()
                            .on_click(|_, _, cx| forward(cx)),
                    ),
                )
            })
            .child(div().flex_1().child(
                div()
                    .text_sm()
                    .child("V1per Servicing Toolkit"),
            ))
            .child(
                div().flex().items_center().gap_px().child(
                    Button::new("minimize")
                        .icon("icons/minimize.svg")
                        .ghost()
                        .small(),
                )
                .child(
                    Button::new("maximize")
                        .icon("icons/maximize.svg")
                        .ghost()
                        .small(),
                )
                .child(
                    Button::new("close")
                        .icon("icons/x.svg")
                        .ghost()
                        .small(),
                ),
            )
    }
}