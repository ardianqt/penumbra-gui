use gpui::prelude::*;
use gpui::{Context, Entity, Render, SharedString, Window, div, px, svg};
use state::{LogLevel, OutputLog};
use ui::{
    ActiveTheme as _, Button, Checkbox, Input, Scroller, Switch, Text,
};

pub(crate) struct MediatekSettings {
    backend: Backend,
    da_path: Option<SharedString>,
    preloader_path: Option<SharedString>,
    auth_path: Option<SharedString>,
    backup_dir: Option<SharedString>,
    log: Entity<OutputLog>,
    log_added: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Backend {
    Auto,
    Libusb,
    Usb,
    Serial,
}

impl MediatekSettings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();

        Self {
            backend: Backend::Auto,
            da_path: None,
            preloader_path: None,
            auth_path: None,
            backup_dir: None,
            log,
            log_added: false,
        }
    }

    fn ensure_log(&mut self, cx: &mut Context<Self>) {
        if !self.log_added {
            self.log_added = true;
            self.log.update(cx, |log, cx| {
                log.push(LogLevel::Info, "Mediatek Settings page loaded", cx);
                log.push(LogLevel::Info, "Backend: Auto (Recommended)", cx);
            });
        }
    }

    fn set_backend(&mut self, backend: Backend, cx: &mut Context<Self>) {
        self.backend = backend;
        let name = match backend {
            Backend::Auto => "Auto (Recommended)",
            Backend::Libusb => "Libusb (Direct USB)",
            Backend::Usb => "nusb (WinUSB / Modern USB)",
            Backend::Serial => "Serial (Virtual COM Port)",
        };
        self.log.update(cx, |log, cx| {
            log.push(LogLevel::Info, format!("Backend changed to: {}", name), cx);
        });
        cx.notify();
    }
}

impl Render for MediatekSettings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_log(cx);
        let theme = *cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .p_6()
            .child(div().text_2xl().child("Mediatek Settings"))
            .child(
                div()
                    .mt_6()
                    .child(div().text_lg().child("Hardware Port Driver Backend"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .mt_3()
                            .gap_2()
                            .children([
                                backend_option(cx, "Auto (Recommended)", "Automatically detects connected BROM/Preloader port", self.backend == Backend::Auto,
                                    cx.listener(move |this: &mut MediatekSettings, _, _, cx| this.set_backend(Backend::Auto, cx))),
                                backend_option(cx, "Libusb (Direct USB)", "Low-level async USB bulk transfers", self.backend == Backend::Libusb,
                                    cx.listener(move |this: &mut MediatekSettings, _, _, cx| this.set_backend(Backend::Libusb, cx))),
                                backend_option(cx, "nusb (WinUSB / Modern USB)", "Cross-platform pure user-mode USB stack", self.backend == Backend::Usb,
                                    cx.listener(move |this: &mut MediatekSettings, _, _, cx| this.set_backend(Backend::Usb, cx))),
                                backend_option(cx, "Serial (Virtual COM Port)", "Connects via /dev/ttyUSB or COM ports", self.backend == Backend::Serial,
                                    cx.listener(move |this: &mut MediatekSettings, _, _, cx| this.set_backend(Backend::Serial, cx))),
                            ]),
                    ),
            )
            .child(
                div()
                    .mt_8()
                    .child(div().text_lg().child("Binary & Authentication Overrides"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .mt_3()
                            .gap_3()
                            .child(path_row(cx, "Custom DA File:", &self.da_path, "Embedded default (da.bin)"))
                            .child(path_row(cx, "Preloader File:", &self.preloader_path, "Auto-detected from scatter"))
                            .child(path_row(cx, "SLA Auth File:", &self.auth_path, "None (bypass enabled)"))
                            .child(path_row(cx, "ROM Backup Directory:", &self.backup_dir, "~/penumbra_backup")),
                    ),
            )
    }
}

fn backend_option(
    cx: &mut Context<MediatekSettings>,
    label: &'static str,
    detail: &'static str,
    selected: bool,
    on_click: impl Fn(&mut MediatekSettings, &mut Window, &mut Context<MediatekSettings>) + 'static,
) -> AnyElement {
    let theme = *cx.theme();
    let accent = match selected {
        true => theme.accent,
        false => theme.muted_foreground,
    };

    div()
        .flex()
        .items_center()
        .gap_3()
        .py_1()
        .cursor_pointer()
        .on_click(cx.listener(on_click))
        .child(
            div()
                .flex_none()
                .size(px(18.))
                .rounded_full()
                .border_2()
                .border_color(accent)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    match selected {
                        true => div()
                            .size(px(10.))
                            .rounded_full()
                            .bg(accent)
                            .into_any_element(),
                        false => div().into_any_element(),
                    },
                ),
        )
        .child(
            div()
                .flex_col()
                .child(div().text_sm().text_color(match selected { true => theme.foreground, false => theme.muted_foreground }).child(label))
                .child(div().text_xs().text_color(theme.muted_foreground).child(detail)),
        )
        .into_any_element()
}

fn path_row(
    cx: &mut Context<MediatekSettings>,
    label: &'static str,
    current: &Option<SharedString>,
    default: &'static str,
) -> AnyElement {
    let theme = *cx.theme();
    let is_default = current.is_none();
    let badge = match is_default {
        true => "DEFAULT",
        false => "OVERRIDE",
    };
    let badge_color = match is_default {
        true => theme.muted_foreground,
        false => theme.warning,
    };

    div()
        .flex()
        .items_center()
        .gap_3()
        .py_2()
        .child(div().w(px(160.)).text_sm().child(label))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .flex_1()
                .child(
                    div()
                        .px_2()
                        .py(px(2.))
                        .rounded_sm()
                        .bg(badge_color.opacity(0.2))
                        .text_xs()
                        .text_color(badge_color)
                        .child(badge),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(theme.foreground)
                        .child(match current {
                            Some(p) => p.as_ref(),
                            None => default,
                        }),
                ),
        )
        .child(
            Button::new((label, "browse"))
                .label("Browse...")
                .ghost()
                .small(),
        )
        .child(
            Button::new((label, "reset"))
                .label("Reset")
                .ghost()
                .small()
                .when(is_default, |b| b.disabled(true)),
        )
        .into_any_element()
}