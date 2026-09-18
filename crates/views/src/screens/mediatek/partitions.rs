use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{AnyElement, Context, Entity, Render, ScrollHandle, SharedString, Task, Window, div, px};
use state::{OutputLog, Io};
use ui::{ActiveTheme as _, Button, Checkbox, Scroller};

use crate::screens::mediatek::flasher::format_size;

pub(crate) struct MediatekPartitions {
    log: Entity<OutputLog>,
    io: Io,
    output_dir: Option<SharedString>,
    partitions: Vec<PartitionEntry>,
    connected: bool,
    task: Option<Task<()>>,
    scrollbar: Entity<ui::Scrollbar>,
}

#[derive(Clone)]
struct PartitionEntry {
    index: usize,
    name: String,
    enabled: bool,
    start_addr: u64,
    size: u64,
    is_critical: bool,
}

impl MediatekPartitions {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let log = OutputLog::global(cx);
        let io = Io::global(cx);
        cx.observe(&log, |_, _, cx| cx.notify()).detach();

        Self {
            log,
            io,
            output_dir: None,
            partitions: Vec::new(),
            connected: false,
            task: None,
            scrollbar: cx.new(|_| ui::Scrollbar::new(ScrollHandle::new())),
        }
    }

    fn dump_partition(&mut self, name: &str, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let io = self.io.clone();
        let part_name = name.to_string();
        let out_dir = self.output_dir.clone()
            .map(|p| PathBuf::from(p.as_ref()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        self.task = Some(cx.spawn(async move |this, cx| {
            let output = out_dir.join(format!("{part_name}.bin"));
            log.update(cx, |l, cx| {
                l.push(LogLevel::Info, format!("Dumping {part_name}..."), cx);
            });

            let result = io.spawn_blocking(move || {
                let da = Vec::new();
                let (mut device, _info) = crate::backend::MtkConnection::connect(&da)?;
                crate::backend::MtkConnection::read_partition(&mut device, &part_name, &output)
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

    fn erase_partition(&mut self, name: &str, cx: &mut Context<Self>) {
        let log = self.log.clone();
        let io = self.io.clone();
        let part_name = name.to_string();

        self.task = Some(cx.spawn(async move |this, cx| {
            log.update(cx, |l, cx| {
                l.push(LogLevel::Warn, format!("Erasing {part_name}..."), cx);
            });

            let result = io.spawn_blocking(move || {
                let da = Vec::new();
                let (mut device, _info) = crate::backend::MtkConnection::connect(&da)?;
                crate::backend::MtkConnection::erase_partition(&mut device, &part_name)
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

    fn toggle_partition(&mut self, index: usize) {
        if let Some(entry) = self.partitions.get_mut(index) {
            entry.enabled = !entry.enabled;
        }
    }

    fn select_all(&mut self) {
        for entry in &mut self.partitions {
            entry.enabled = true;
        }
    }

    fn deselect_all(&mut self) {
        for entry in &mut self.partitions {
            entry.enabled = false;
        }
    }
}

impl Render for MediatekPartitions {
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
                    .child(div().text_sm().child("Output Folder:"))
                    .child({
                        let label = self.output_dir.clone().unwrap_or_else(|| "Not selected".into());
                        div().flex_1().text_color(theme.muted_foreground).text_sm().child(label)
                    })
                    .child(Button::new("browse-output").label("Browse...").ghost().small())
                    .child(div().flex_1())
                    .child(Button::new("backup-selected").label("Backup Selected").ghost().small())
                    .child(Button::new("backup-all").label("Backup All").ghost().small()),
            )
            .child(
                div().flex().items_center().px_4().py_1().gap_2().border_b_1().border_color(theme.sidebar_border)
                    .child(Button::new("part-select-all").label("Select All").ghost().small()
                        .on_click(cx.listener(|this, _, _, cx| { this.select_all(); cx.notify(); })))
                    .child(Button::new("part-deselect-all").label("Deselect All").ghost().small()
                        .on_click(cx.listener(|this, _, _, cx| { this.deselect_all(); cx.notify(); })))
                    .child(div().flex_1())
                    .child(Button::new("refresh-table").label("Refresh").ghost().small()),
            )
            .child(
                div().flex().items_center().px_4().py_1().gap_4().bg(theme.sidebar)
                    .border_b_1().border_color(theme.sidebar_border).text_sm().text_color(theme.muted_foreground)
                    .child(div().w(px(36.)).child(""))
                    .child(div().w(px(40.)).child("#"))
                    .child(div().w(px(150.)).child("Partition"))
                    .child(div().w(px(100.)).child("Start Addr"))
                    .child(div().w(px(100.)).child("Size"))
                    .child(div().flex_1().child("Operations")),
            )
            .child(
                Scroller::new("partitions-list", &self.scrollbar).flex_1().child(
                    if self.partitions.is_empty() {
                        div().flex().items_center().justify_center().h_full()
                            .text_color(theme.muted_foreground).text_sm()
                            .child("Connect a device and load a scatter file to see partitions.")
                            .into_any_element()
                    } else {
                        let parts: Vec<_> = self.partitions.iter().map(|p| {
                            (p.index, p.name.clone(), p.enabled, p.start_addr, p.size)
                        }).collect();
                        div().flex().flex_col().w_full().children(parts.into_iter().map(|(idx, name, enabled, start_addr, size)| {
                            let name_clone = name.clone();
                            div().flex().items_center().px_4().py(px(6.)).gap_4()
                                .border_b_1().border_color(theme.sidebar_border.opacity(0.5)).text_sm()
                                .child(Checkbox::new(("part-entry", idx), enabled)
                                    .on_click(cx.listener(move |this, _, _, _| { this.toggle_partition(idx); })))
                                .child(div().w(px(40.)).text_color(theme.muted_foreground).child(idx.to_string()))
                                .child(div().w(px(150.)).flex().items_center().gap_1()
                                    .child(div().child(name.clone())))
                                .child(div().w(px(100.)).child(format!("0x{:08X}", start_addr)))
                                .child(div().w(px(100.)).child(format_size(size)))
                                .child(div().flex_1().flex().items_center().gap_2()
                                    .child(Button::new(("dump", idx)).label("Dump").ghost().small()
                                        .on_click(cx.listener(move |this, _, _, cx| this.dump_partition(&name_clone, cx))))
                                    .child(Button::new(("erase", idx)).label("Erase").ghost().small()
                                        .on_click(cx.listener(move |this, _, _, cx| this.erase_partition(&name_clone, cx)))))
                                .into_any_element()
                        })),
                    },
                ),
            )
    }
}
