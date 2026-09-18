use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{AnyElement, Context, Entity, Render, ScrollHandle, SharedString, Task, Window, div, px};
use state::{LogLevel, OutputLog, Io};
use ui::{
    ActiveTheme as _, Button, Checkbox, Input, Scroller,
};

use crate::backend::{MtkConnection, ScatterEntry};

pub(crate) struct MediatekFlasher {
    log: Entity<OutputLog>,
    io: Io,
    scatter_path: Option<SharedString>,
    scatter_entries: Vec<ScatterEntry>,
    search_input: Entity<Input>,
    connected: bool,
    device_name: Option<SharedString>,
    da_bytes: Vec<u8>,
    task: Option<Task<()>>,
    scrollbar: Entity<ui::Scrollbar>,
}

impl MediatekFlasher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        let io = Io::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();

        let search_input = cx.new(|cx| {
            Input::new("", cx)
                .icon("icons/search.svg")
                .compact()
        });

        Self {
            log,
            io,
            scatter_path: None,
            scatter_entries: Vec::new(),
            search_input,
            connected: false,
            device_name: None,
            da_bytes: Vec::new(),
            task: None,
            scrollbar: cx.new(|_| ui::Scrollbar::new(ScrollHandle::new())),
        }
    }

    fn connect(&mut self, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let da = self.da_bytes.clone();
        let io = self.io.clone();

        self.task = Some(cx.spawn(async move |this, cx| {
            log.update(cx, |l, cx| l.push(LogLevel::Info, "Connecting to device...", cx));

            let result = io.spawn_blocking(move || {
                MtkConnection::connect(&da)
            }).await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(Ok((device, info))) => {
                        this.connected = true;
                        this.device_name = Some(info.chip_name.clone().into());
                        log.update(cx, |l, cx| {
                            l.push(LogLevel::Info, format!("Connected: {}", info.chip_name), cx);
                            l.push(LogLevel::Info, format!("HW Code: 0x{:08X}", info.hw_code), cx);
                            if info.sbc { l.push(LogLevel::Info, "SBC: enabled", cx); }
                            if info.sla { l.push(LogLevel::Info, "SLA: enabled", cx); }
                            if info.daa { l.push(LogLevel::Info, "DAA: enabled", cx); }
                        });
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

    fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.connected = false;
        self.device_name = None;
        self.log.update(cx, |l, cx| {
            l.push(LogLevel::Info, "Disconnected", cx);
        });
        cx.notify();
    }

    fn load_scatter(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let io = self.io.clone();

        self.task = Some(cx.spawn(async move |this, cx| {
            let result = io.spawn_blocking(move || {
                MtkConnection::parse_scatter(&path)
            }).await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(entries)) => {
                        let count = entries.len();
                        this.scatter_path = Some(path.to_string_lossy().to_string().into());
                        this.scatter_entries = entries;
                        log.update(cx, |l, cx| {
                            l.push(LogLevel::Info, format!("Loaded scatter: {count} partitions found"), cx);
                        });
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

    fn toggle_partition(&mut self, index: usize) {
        if let Some(entry) = self.scatter_entries.get_mut(index) {
            entry.enabled = !entry.enabled;
        }
    }

    fn select_all(&mut self) {
        for entry in &mut self.scatter_entries {
            entry.enabled = true;
        }
    }

    fn deselect_all(&mut self) {
        for entry in &mut self.scatter_entries {
            entry.enabled = false;
        }
    }

    fn firmware_only(&mut self) {
        let skip = ["userdata", "nvram", "protect_f", "protect_s", "secfg"];
        for entry in &mut self.scatter_entries {
            entry.enabled = !skip.iter().any(|s| entry.name.to_lowercase().contains(s));
        }
    }

    fn flash(&mut self, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let io = self.io.clone();
        let scatter = self.scatter_path.clone();
        let selected: Vec<String> = self.scatter_entries.iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.clone())
            .collect();
        let da = self.da_bytes.clone();

        if selected.is_empty() {
            log.update(cx, |l, cx| l.push(LogLevel::Warn, "No partitions selected", cx));
            return;
        }

        self.task = Some(cx.spawn(async move |this, cx| {
            log.update(cx, |l, cx| {
                l.push(LogLevel::Info, format!("Flashing {} partitions...", selected.len()), cx);
            });

            let result = io.spawn_blocking(move || {
                let (mut device, _info) = MtkConnection::connect(&da)?;
                let scatter_path = PathBuf::from(scatter.unwrap_or_default());
                MtkConnection::flash_scatter(&mut device, &scatter_path, &selected)
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

impl Render for MediatekFlasher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_2()
                    .gap_2()
                    .bg(theme.sidebar)
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .child(div().text_sm().child("Device:"))
                    .child({
                        let name = self.device_name.clone().unwrap_or_else(|| "No device connected".into());
                        div()
                            .flex_1()
                            .text_color(if self.connected { theme.primary } else { theme.muted_foreground })
                            .text_sm()
                            .child(name)
                    })
                    .child(if self.connected {
                        Button::new("disconnect").label("Disconnect").ghost().small()
                            .on_click(cx.listener(|this, _, _, cx| this.disconnect(cx)))
                    } else {
                        Button::new("connect")
                            .label("Connect")
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| this.connect(cx)))
                    })
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_2()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .child(div().text_sm().child("Scatter:"))
                    .child({
                        let label = self.scatter_path.clone().unwrap_or_else(|| "No scatter file loaded".into());
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .text_sm()
                            .child(label)
                    })
                    .child(Button::new("browse-scatter").label("Browse").ghost().small()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_2()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .child(Button::new("select-all").label("Select All").ghost().small()
                        .on_click(cx.listener(|this, _, _, cx| { this.select_all(); cx.notify(); })))
                    .child(Button::new("deselect-all").label("Deselect All").ghost().small()
                        .on_click(cx.listener(|this, _, _, cx| { this.deselect_all(); cx.notify(); })))
                    .child(Button::new("skip-userdata").label("Firmware Only").ghost().small()
                        .on_click(cx.listener(|this, _, _, cx| { this.firmware_only(); cx.notify(); })))
                    .child(div().flex_1())
                    .child(self.search_input.clone())
                    .child(
                        Button::new("flash-selected")
                            .label(format!("Flash ({})", self.scatter_entries.iter().filter(|e| e.enabled).count()))
                            .ghost()
                            .small()
                            .when(!self.connected, |b| b.disabled(true))
                            .on_click(cx.listener(|this, _, _, cx| this.flash(cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .gap_4()
                    .bg(theme.sidebar)
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(div().w(px(36.)).child(""))
                    .child(div().w(px(140.)).child("Partition"))
                    .child(div().w(px(160.)).child("Image File"))
                    .child(div().w(px(100.)).child("Start Addr"))
                    .child(div().w(px(100.)).child("Size"))
                    .child(div().flex_1().child("Status")),
            )
            .child(
                Scroller::new("partition-list", &self.scrollbar)
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .children(self.scatter_entries.iter().enumerate().map(|(i, p)| {
                                let addr = format!("0x{:08X}", p.offset);
                                let size_str = format_size(p.size);

                                div()
                                    .flex()
                                    .items_center()
                                    .px_4()
                                    .py(px(6.))
                                    .gap_4()
                                    .border_b_1()
                                    .border_color(theme.sidebar_border.opacity(0.5))
                                    .text_sm()
                                    .child(Checkbox::new(("part-check", i), p.enabled)
                                        .on_click(cx.listener(move |this, _, _, _| { this.toggle_partition(i); })))
                                    .child(
                                        div().w(px(140.)).flex().items_center().gap_1()
                                            .child(div().child(p.name.clone())),
                                    )
                                    .child(div().w(px(160.)).text_color(theme.muted_foreground).child(
                                        if p.file_name.is_empty() { "-" } else { &p.file_name },
                                    ))
                                    .child(div().w(px(100.)).child(addr))
                                    .child(div().w(px(100.)).child(size_str))
                                    .child(div().flex_1().text_color(theme.muted_foreground).child(
                                        if p.enabled { "Selected" } else { "" },
                                    ))
                                    .into_any_element()
                            })),
                    ),
            )
    }
}

pub(crate) fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024. * 1024. * 1024.))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024. * 1024.))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.)
    } else {
        format!("{} B", bytes)
    }
}
