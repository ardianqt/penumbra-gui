use gpui::{App, Context, Entity, Global, SharedString};
use std::collections::VecDeque;

#[derive(Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
}

pub struct OutputLog {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    filter: LogFilter,
    auto_scroll: bool,
}

impl OutputLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(100),
            max_entries: 5000,
            filter: LogFilter::All,
            auto_scroll: true,
        }
    }

    pub fn push(&mut self, level: LogLevel, message: impl Into<String>, cx: &mut Context<Self>) {
        let entry = LogEntry {
            level,
            message: message.into(),
            timestamp: chrono::now(),
        };
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        cx.notify();
    }

    pub fn entries(&self) -> impl Iterator<Item = &LogEntry> {
        let filter = self.filter;
        self.entries
            .iter()
            .filter(move |e| filter.matches(&e.level))
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.entries.clear();
        cx.notify();
    }

    pub fn set_filter(&mut self, filter: LogFilter, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }

    pub fn filter(&self) -> LogFilter {
        self.filter
    }

    pub fn set_auto_scroll(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.auto_scroll = enabled;
        cx.notify();
    }

    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogFilter {
    All,
    Errors,
    Warnings,
    Info,
}

impl LogFilter {
    pub fn matches(&self, level: &LogLevel) -> bool {
        match self {
            Self::All => true,
            Self::Errors => matches!(level, LogLevel::Error),
            Self::Warnings => matches!(level, LogLevel::Warn),
            Self::Info => matches!(level, LogLevel::Info),
        }
    }
}

struct OutputLogHandle(Entity<OutputLog>);

impl Global for OutputLogHandle {}

impl OutputLog {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<OutputLogHandle>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let log = cx.new(|_| OutputLog::new());
        cx.set_global(OutputLogHandle(log.clone()));
        log
    }
}

// Placeholder - in real build this would use jiff or chrono
fn chrono() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}