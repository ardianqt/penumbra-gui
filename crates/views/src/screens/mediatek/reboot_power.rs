use gpui::prelude::*;
use gpui::{Context, Entity, Render, Window, div, px};
use state::{LogLevel, OutputLog};
use ui::{
    ActiveTheme as _, Button, Card, Separator, Text,
};

pub(crate) struct MediatekRebootPower {
    log: Entity<OutputLog>,
    log_added: bool,
}

impl MediatekRebootPower {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();
        Self {
            log,
            log_added: false,
        }
    }

    fn ensure_log(&mut self, cx: &mut Context<Self>) {
        if !self.log_added {
            self.log_added = true;
            self.log.update(cx, |log, cx| {
                log.push(LogLevel::Info, "Reboot & Power page loaded", cx);
            });
        }
    }

    fn execute(&mut self, action: &str, cx: &mut Context<Self>) {
        self.log.update(cx, |log, cx| {
            log.push(LogLevel::Info, format!("Executing: {}", action), cx);
        });
    }
}

impl Render for MediatekRebootPower {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_log(cx);
        let theme = *cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .p_6()
            .child(
                div()
                    .text_2xl()
                    .child("Reboot & Power"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .mt_6()
                    .child(tile_card(cx, "Normal Boot", theme.accent, "Reboot device into standard Android OS",
                        cx.listener(|this: &mut MediatekRebootPower, _, _, cx| this.execute("Normal Boot", cx))))
                    .child(tile_card(cx, "Fastboot", theme.foreground, "Reboot into MediaTek Fastboot bootloader",
                        cx.listener(|this: &mut MediatekRebootPower, _, _, cx| this.execute("Fastboot", cx))))
                    .child(tile_card(cx, "Meta Mode", theme.foreground, "Reboot into Factory Meta mode for calibration",
                        cx.listener(|this: &mut MediatekRebootPower, _, _, cx| this.execute("Meta Mode", cx))))
                    .child(tile_card(cx, "Power Off", theme.error, "Shut down device and release hardware bus",
                        cx.listener(|this: &mut MediatekRebootPower, _, _, cx| this.execute("Power Off", cx)))),
            )
    }
}

fn tile_card(
    cx: &mut Context<MediatekRebootPower>,
    title: &'static str,
    accent: gpui::Hsla,
    description: &'static str,
    on_execute: impl Fn(&mut MediatekRebootPower, &mut Window, &mut Context<MediatekRebootPower>) + 'static,
) -> AnyElement {
    let theme = *cx.theme();

    div()
        .flex()
        .flex_col()
        .w(px(220.))
        .bg(theme.sidebar)
        .rounded_lg(theme.radius)
        .border_1()
        .border_color(theme.sidebar_border)
        .p_4()
        .child(
            div()
                .text_lg()
                .text_color(accent)
                .child(title),
        )
        .child(
            div()
                .mt_2()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(description),
        )
        .child(
            div()
                .mt_4()
                .child(
                    Button::new(title)
                        .label("Execute")
                        .w_full()
                        .ghost()
                        .on_click(cx.listener(on_execute)),
                ),
        )
        .into_any_element()
}