use gpui::prelude::*;
use gpui::{Context, Entity, Pixels, Render, ScrollHandle, Window, div, px, svg};
use state::{LogFilter, LogLevel, OutputLog, SideTab};
use ui::{ActiveTheme as _, Button, Panel, Scroller, Side};

const MIN_WIDTH: Pixels = px(240.);
const MAX_WIDTH: Pixels = px(560.);

pub(crate) struct SidebarRight {
    width: Pixels,
    open: bool,
    log: Entity<OutputLog>,
    scrollbar: Entity<ui::Scrollbar>,
}

impl SidebarRight {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();

        Self {
            width: px(240.).clamp(MIN_WIDTH, MAX_WIDTH),
            open: false,
            log,
            scrollbar: cx.new(|_| ui::Scrollbar::new(ScrollHandle::new())),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn available(window: &Window) -> bool {
        window.viewport_size().width > px(800.)
    }

    pub fn covers_content(&self, _window: &Window) -> bool {
        false
    }

    pub fn occupied_width(&self, window: &Window) -> Pixels {
        match self.open && Self::available(window) {
            false => Pixels::ZERO,
            true => self.width,
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    pub fn show(&mut self, _tab: SideTab, cx: &mut Context<Self>) {
        self.open = true;
        cx.notify();
    }
}

impl Render for SidebarRight {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let log = self.log.read(cx);
        let filter = log.filter();
        let auto_scroll = log.auto_scroll();

        let entries: Vec<state::LogEntry> = log.entries().cloned().collect();
        drop(log);

        Panel::new("sidebar-right", Side::Right, self.width)
            .limits(MIN_WIDTH, MAX_WIDTH)
            .when(!self.open, |this| this.hidden())
            .bg(theme.sidebar)
            .border_color(theme.sidebar_border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.sidebar_border)
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .child("Output Log"),
                            )
                            .child(
                                Button::new("clear-log")
                                    .label("Clear")
                                    .ghost()
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.log.update(cx, |log, cx| log.clear(cx));
                                    })),
                            ),
                    )
                    // Filter bar
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px_3()
                            .py_1()
                            .gap_1()
                            .border_b_1()
                            .border_color(theme.sidebar_border.opacity(0.5))
                            .child(filter_button(cx, "All", filter == LogFilter::All, move || {
                                LogFilter::All
                            }))
                            .child(filter_button(cx, "Errors", filter == LogFilter::Errors, move || {
                                LogFilter::Errors
                            }))
                            .child(filter_button(cx, "Warnings", filter == LogFilter::Warnings, move || {
                                LogFilter::Warnings
                            }))
                            .child(filter_button(cx, "Info", filter == LogFilter::Info, move || {
                                LogFilter::Info
                            }))
                            .child(div().flex_1())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(
                                        svg()
                                            .path(icons::path(match auto_scroll {
                                                true => "icons/chevron-down.svg",
                                                false => "icons/chevron-up.svg",
                                            }))
                                            .size(px(12.))
                                            .text_color(theme.muted_foreground),
                                    )
                                    .child("Auto-scroll"),
                            ),
                    )
                    // Log entries
                    .child(
                        Scroller::new("output-log-scroll", &self.scrollbar)
                            .flex_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .px_2()
                                    .py_1()
                                    .gap_px()
                                    .children(
                                        if entries.is_empty() {
                                            vec![div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .h_full()
                                                .text_color(theme.muted_foreground)
                                                .text_xs()
                                                .child("No log entries")
                                                .into_any_element()]
                                        } else {
                                            entries.iter().map(|entry| {
                                                let color = match entry.level {
                                                    LogLevel::Error => theme.danger,
                                                    LogLevel::Warn => theme.secondary,
                                                    LogLevel::Info => theme.primary,
                                                    LogLevel::Debug => theme.muted_foreground,
                                                };
                                                div()
                                                    .flex()
                                                    .flex_none()
                                                    .gap_2()
                                                    .py(px(2.))
                                                    .px_1()
                                                    .text_xs()
                                                    .child(
                                                        div()
                                                            .flex_none()
                                                            .font_family("monospace")
                                                            .text_color(theme.muted_foreground.opacity(0.7))
                                                            .child(entry.timestamp.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family("monospace")
                                                            .text_color(color)
                                                            .child(entry.message.clone()),
                                                    )
                                                    .into_any_element()
                                            }).collect()
                                        },
                                    ),
                            ),
                    ),
            )
    }
}

fn filter_button(
    cx: &mut Context<SidebarRight>,
    label: &'static str,
    active: bool,
    filter: fn() -> LogFilter,
) -> AnyElement {
    let theme = *cx.theme();

    Button::new(label)
        .label(label)
        .ghost()
        .small()
        .when(active, |b| {
            b.text_color(theme.primary)
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            this.log.update(cx, |log, cx| log.set_filter(filter(), cx));
        }))
        .into_any_element()
}