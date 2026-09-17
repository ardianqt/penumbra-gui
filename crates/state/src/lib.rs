mod logging;
mod output_log;
mod session;
mod settings;
mod toast;
mod window_shape;

pub use logging::log_file;
pub use output_log::{LogEntry, LogFilter, LogLevel, OutputLog};
pub use session::{Session, SessionEvent, SessionState};
pub use settings::{
    AppSettings, FullscreenControlsAutohide, SideTab, remember_window, window_placement,
};
pub use toast::{Outcome, Target, Toast, Toasts};
pub use window_shape::{apply_window_rounding, install_rounded_window_hook};

use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use gpui::{App, AppContext as _, Entity, Global};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct Io(Arc<Runtime>);

impl Global for Io {}

impl Io {
    pub fn new() -> Result<Self> {
        Ok(Self(Arc::new(Runtime::new()?)))
    }

    pub fn global(cx: &App) -> Self {
        cx.global::<Self>().clone()
    }

    pub fn handle(&self) -> tokio::runtime::Handle {
        self.0.handle().clone()
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }

    pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.0.spawn_blocking(func)
    }
}

pub(crate) async fn join<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    handle.await?
}

pub struct Sonora {
    pub session: Entity<Session>,
    pub settings: Entity<AppSettings>,
}

impl Global for Sonora {}

impl Sonora {
    pub fn global(cx: &App) -> &Self {
        cx.global()
    }
}

pub fn init(cx: &mut App, io: Io) {
    cx.set_global(io.clone());

    let settings = cx.new(|_| AppSettings::load());
    let session = cx.new(|cx| Session::new(cx));
    let log = OutputLog::init(cx);

    cx.set_global(Sonora { session, settings });
}