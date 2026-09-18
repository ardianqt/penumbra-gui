use gpui::prelude::*;
use gpui::{Context, Entity, Render, ScrollHandle, SharedString, Task, Window, div, px};
use state::{LogLevel, OutputLog, Io};
use ui::{
    ActiveTheme as _, Button, Checkbox, Input, Scroller,
};

use crate::backend::MtkConnection;

pub(crate) struct MediatekFlasher {
    log: Entity<OutputLog>,
    io: Io,
    scatter_path: Option<SharedString>,
    search_input: Entity<Input>,
    partitions: Vec<PartitionRow>,
    connected: bool,
    device_name: Option<SharedString>,
    da_bytes: Vec<u8>,
    task: Option<Task<()>>,
    scrollbar: Entity<ui::Scrollbar>,
}

#[derive(Clone)]
struct PartitionRow {
    name: String,
    enabled: bool,
    image_path: Option<SharedString>,
    start_addr: u64,
    size: u64,
    is_bootloader: bool,
    status: PartitionStatus,
}

#[derive(Clone, PartialEq)]
enum PartitionStatus {
    Unassigned,
    Ready,
    Flashing,
    Done,
    Failed,
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

        let partitions = vec![
            ("preloader", 0x0, 0x200000, true, PartitionStatus::Unassigned),
            ("lk", 0x200000, 0x400000, true, PartitionStatus::Unassigned),
            ("boot", 0x600000, 0x800000, false, PartitionStatus::Unassigned),
            ("recovery", 0xE00000, 0x800000, false, PartitionStatus::Unassigned),
            ("system", 0x1600000, 0x8000000, false, PartitionStatus::Unassigned),
            ("vendor", 0x9600000, 0x2000000, false, PartitionStatus::Unassigned),
            ("userdata", 0xB600000, 0x40000000, false, PartitionStatus::Unassigned),
        ].into_iter().map(|(n, s, sz, bl, st)| PartitionRow {
            name: n.to_string(),
            enabled: true,
            image_path: None,
            start_addr: s,
            size: sz,
            is_bootloader: bl,
            status: st,
        }).collect();

        Self {
            log,
            io,
            scatter_path: None,
            search_input,
            partitions,
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
                    Ok(Ok((info, msg))) => {
                        this.connected = true;
                        this.device_name = Some(info.chip_name.clone().into());
                        log.update(cx, |l, cx| {
                            l.push(LogLevel::Info, msg, cx);
                            l.push(LogLevel::Info, format!("HW Code: 0x{:08X}", info.hw_code), cx);
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
                    .child(
                        div()
                            .flex_1()
                            .text_color(match self.connected { true => theme.primary, false => theme.muted_foreground })
                            .text_sm()
                            .child(match &self.device_name {
                                Some(n) => n.as_ref(),
                                None => "No device connected",
                            }),
                    )
                    .child(match self.connected {
                        true => Button::new("disconnect").label("Disconnect").ghost().small(),
                        false => Button::new("connect")
                            .label("Connect")
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| this.connect(cx))),
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
                    .child(div().text_sm().child("Scatter File:"))
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .text_sm()
                            .child(match &self.scatter_path {
                                Some(p) => p.as_ref(),
                                None => "No scatter file loaded",
                            }),
                    )
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
                    .child(Button::new("select-all").label("Select All").ghost().small())
                    .child(Button::new("deselect-all").label("Deselect All").ghost().small())
                    .child(Button::new("skip-userdata").label("Firmware Only").ghost().small())
                    .child(div().flex_1())
                    .child(self.search_input.clone())
                    .child(Button::new("console-toggle").label("Console").ghost().small())
                    .child(
                        Button::new("flash-selected")
                            .label("Flash Selected")
                            .ghost()
                            .small()
                            .when(!self.connected, |b| b.disabled(true)),
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
                    .child(div().w(px(160.)).child("Target Image"))
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
                            .children(self.partitions.iter().enumerate().map(|(i, p)| {
                                let addr = format!("0x{:08X}", p.start_addr);
                                let size_str = format_size(p.size);
                                let (status_text, status_color) = match p.status {
                                    PartitionStatus::Unassigned => ("Unassigned", theme.muted_foreground),
                                    PartitionStatus::Ready => ("Ready", theme.primary),
                                    PartitionStatus::Flashing => ("Flashing...", theme.secondary),
                                    PartitionStatus::Done => ("Done", theme.primary),
                                    PartitionStatus::Failed => ("Failed", theme.danger),
                                };
                                let bl_badge = p.is_bootloader.then(|| {
                                    div().px_1().rounded_sm().bg(theme.secondary.opacity(0.2))
                                        .text_color(theme.secondary).text_xs().child("BL").into_any_element()
                                });

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
                                        .on_click(cx.listener(move |_, _, _, _| {})))
                                    .child(
                                        div().w(px(140.)).flex().items_center().gap_1()
                                            .child(div().child(p.name.clone()))
                                            .when_some(bl_badge, |this, badge| this.child(badge)),
                                    )
                                    .child(div().w(px(160.)).text_color(theme.muted_foreground).child(
                                        match &p.image_path { Some(path) => path.as_ref(), None => "-" },
                                    ))
                                    .child(div().w(px(100.)).child(addr))
                                    .child(div().w(px(100.)).child(size_str))
                                    .child(div().flex_1().text_color(status_color).child(status_text))
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