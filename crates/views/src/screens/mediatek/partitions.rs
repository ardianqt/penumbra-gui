use gpui::prelude::*;
use gpui::{Context, Entity, Render, ScrollHandle, SharedString, Task, Window, div, px};
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

        let partitions = vec![
            ("preloader", 0x0, 0x200000, true),
            ("lk", 0x200000, 0x400000, true),
            ("boot", 0x600000, 0x800000, false),
            ("recovery", 0xE00000, 0x800000, false),
            ("system", 0x1600000, 0x8000000, false),
            ("vendor", 0x9600000, 0x2000000, false),
            ("metadata", 0xB600000, 0x200000, true),
            ("nvram", 0xB800000, 0x1000000, false),
            ("userdata", 0xCC00000, 0x40000000, false),
        ].into_iter().enumerate().map(|(i, (n, s, sz, cr))| PartitionEntry {
            index: i + 1,
            name: n.to_string(),
            enabled: false,
            start_addr: s,
            size: sz,
            is_critical: cr,
        }).collect();

        Self { log, io, output_dir: None, partitions, connected: false, task: None, scrollbar: cx.new(|_| ui::Scrollbar::new(ScrollHandle::new())) }
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
                    .child(div().flex_1().text_color(theme.muted_foreground).text_sm()
                        .child(match &self.output_dir { Some(d) => d.as_ref(), None => "Not selected" }))
                    .child(Button::new("browse-output").label("Browse...").ghost().small())
                    .child(div().flex_1())
                    .child(Button::new("backup-selected").label("Backup Selected").ghost().small())
                    .child(Button::new("backup-all").label("Backup All").ghost().small()),
            )
            .child(
                div()
                    .flex().items_center().px_4().py_1().gap_2().border_b_1().border_color(theme.sidebar_border)
                    .child(Button::new("part-select-all").label("Select All").ghost().small())
                    .child(Button::new("part-deselect-all").label("Deselect All").ghost().small())
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
                    div().flex().flex_col().w_full().children(self.partitions.iter().map(|p| {
                        let critical_badge = p.is_critical.then(|| {
                            div().px_1().rounded_sm().bg(theme.danger.opacity(0.2)).text_color(theme.danger)
                                .text_xs().child("CRIT").into_any_element()
                        });
                        div().flex().items_center().px_4().py(px(6.)).gap_4()
                            .border_b_1().border_color(theme.sidebar_border.opacity(0.5)).text_sm()
                            .child(Checkbox::new(("part-entry", p.index), p.enabled)
                                .on_click(cx.listener(move |_, _, _, _| {})))
                            .child(div().w(px(40.)).text_color(theme.muted_foreground).child(p.index.to_string()))
                            .child(div().w(px(150.)).flex().items_center().gap_1()
                                .child(div().child(p.name.clone()))
                                .when_some(critical_badge, |this, badge| this.child(badge)))
                            .child(div().w(px(100.)).child(format!("0x{:08X}", p.start_addr)))
                            .child(div().w(px(100.)).child(format_size(p.size)))
                            .child(div().flex_1().flex().items_center().gap_2()
                                .child(Button::new(("dump", p.index)).label("Dump").ghost().small())
                                .child(Button::new(("write", p.index)).label("Write").ghost().small())
                                .child(if p.is_critical {
                                    Button::new(("erase", p.index)).label("Protected").ghost().small().into_any_element()
                                } else {
                                    Button::new(("erase", p.index)).label("Erase").ghost().small().into_any_element()
                                }))
                            .into_any_element()
                    })),
                ),
            )
    }
}