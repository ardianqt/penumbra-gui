use gpui::prelude::*;
use gpui::{AnyElement, Context, Entity, Render, Task, Window, div, px};
use state::{LogLevel, OutputLog, Io};
use ui::{
    ActiveTheme as _, Button,
};

pub(crate) struct MediatekRebootPower {
    log: Entity<OutputLog>,
    io: Io,
    task: Option<Task<()>>,
}

impl MediatekRebootPower {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        let io = Io::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();
        Self {
            log,
            io,
            task: None,
        }
    }

    fn execute(&mut self, action: &'static str, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let io = self.io.clone();

        self.task = Some(cx.spawn(async move |this, cx| {
            log.update(cx, |l, cx| {
                l.push(LogLevel::Info, format!("Sending {action} command..."), cx);
            });

            let result = io.spawn_blocking(move || {
                let da = Vec::new();
                let (mut device, _info) = crate::backend::MtkConnection::connect(&da)?;
                crate::backend::MtkConnection::reboot(&mut device, action)
            }).await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(msg)) => {
                        log.update(cx, |l, cx| l.push(LogLevel::Info, msg, cx));
                    }
                    Ok(Err(e)) => {
                        log.update(cx, |l, cx| l.push(LogLevel::Error, e, cx));
                    }
                    Err(e) => {
                        log.update(cx, |l, cx| l.push(LogLevel::Error, format!("Task failed: {e}"), cx));
                    }
                }
                cx.notify();
            }).ok();
        }));
    }
}

impl Render for MediatekRebootPower {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(
                div()
                    .text_2xl()
                    .child("Reboot & Power"),
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Connect a device first, then select a reboot mode."),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .mt_6()
                    .child(tile_card(cx, "Normal Boot", theme.primary, "Reboot device into standard Android OS", "Normal Boot"))
                    .child(tile_card(cx, "Fastboot", theme.foreground, "Reboot into MediaTek Fastboot bootloader", "Fastboot"))
                    .child(tile_card(cx, "Meta Mode", theme.foreground, "Reboot into Factory Meta mode for calibration", "Meta Mode"))
                    .child(tile_card(cx, "Power Off", theme.danger, "Shut down device and release hardware bus", "Power Off")),
            )
    }
}

fn tile_card(
    cx: &mut Context<MediatekRebootPower>,
    title: &'static str,
    accent: gpui::Hsla,
    description: &'static str,
    action: &'static str,
) -> AnyElement {
    let theme = *cx.theme();

    div()
        .flex()
        .flex_col()
        .w(px(220.))
        .bg(theme.sidebar)
        .rounded_lg()
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
                        .on_click(cx.listener(move |this, _, _, cx| this.execute(action, cx))),
                ),
        )
        .into_any_element()
}
