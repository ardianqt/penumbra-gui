use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context as _, Result};
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use gpui::WindowDecorations;
use gpui::{
    App, Bounds, Context, DisplayId, Pixels, Size, Subscription, Task, Window, WindowBounds, point,
    px, size,
};
use serde::{Deserialize, Serialize};
use ui::{
    Layout, Look, Mode, Pace, Rounding, Saver, Sorting, Stillness, ThemeKind, ThemeOverrides,
};

use crate::Sonora;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SideTab {
    #[default]
    Queue,
    Lyrics,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FullscreenControlsAutohide {
    #[default]
    Automatic,
    AlwaysShown,
    AlwaysHidden,
}

impl FullscreenControlsAutohide {
    pub const ALL: [Self; 3] = [Self::Automatic, Self::AlwaysShown, Self::AlwaysHidden];

    pub fn id(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::AlwaysHidden => "always-hidden",
            Self::AlwaysShown => "always-shown",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "automatic" => Self::Automatic,
            "always-hidden" => Self::AlwaysHidden,
            "always-shown" => Self::AlwaysShown,
            _ => Self::Automatic,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Automatic => "settings-fullscreen-controls-autohide-automatic",
            Self::AlwaysHidden => "settings-fullscreen-controls-autohide-always-hidden",
            Self::AlwaysShown => "settings-fullscreen-controls-autohide-always-shown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Frame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    maximized: bool,
}

impl Frame {
    fn of(window: &Window) -> Self {
        let placement = window.window_bounds();
        let bounds = placement.get_bounds();
        Self {
            x: bounds.origin.x / px(1.),
            y: bounds.origin.y / px(1.),
            width: bounds.size.width / px(1.),
            height: bounds.size.height / px(1.),
            maximized: matches!(placement, WindowBounds::Maximized(_)),
        }
    }

    fn sane(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|it| it.is_finite())
            && self.width > 0.
            && self.height > 0.
    }

    fn placement(self, least: Size<Pixels>) -> WindowBounds {
        let bounds = Bounds {
            origin: point(px(self.x), px(self.y)),
            size: size(
                px(self.width).max(least.width),
                px(self.height).max(least.height),
            ),
        };
        match self.maximized {
            true => WindowBounds::Maximized(bounds),
            false => WindowBounds::Windowed(bounds),
        }
    }
}

fn system_font() -> String {
    SYSTEM_FONT.to_owned()
}

const SAVE_DELAY: Duration = Duration::from_millis(300);
const DEFAULT_VOLUME: f32 = 0.7;
const DEFAULT_SIDEBAR_WIDTH: f32 = 195.;
const DEFAULT_SIDEBAR_RIGHT_WIDTH: f32 = 254.;
const DEFAULT_FONT_SIZE: f32 = 14.;
const DEFAULT_STARTUP: &str = "home";
const SETTINGS_VERSION: u32 = 2;

pub const SYSTEM_FONT: &str = "auto";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
struct Values {
    version: u32,
    language: String,
    #[serde(default = "system_font")]
    font: String,
    startup: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    hidden_nav: BTreeMap<String, bool>,
    appearance: Appearance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
struct Appearance {
    theme: String,
    adaptive_theme: bool,
    icons: String,
    rounding: String,
    blur: bool,
    font_size: f32,
    transparent: bool,
    transparency: f32,
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    server_side_decorations: bool,
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    window_rounding: String,
    window_controls: bool,
    #[cfg(not(target_os = "macos"))]
    traffic_light_controls: bool,
    controls_on_left: bool,
    reduce_motion: String,
    motion_pace: String,
    battery_saver: String,
    theme_overrides: ThemeOverrides,
    fullscreen_controls_autohide: String,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            language: i18n::AUTO.to_owned(),
            font: system_font(),
            startup: DEFAULT_STARTUP.to_owned(),
            hidden_nav: BTreeMap::new(),
            appearance: Appearance::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
struct StateValues {
    volume: f32,
    sidebar_width: f32,
    sidebar_open: bool,
    sidebar_right_width: f32,
    sidebar_right_open: bool,
    sidebar_right_tab: SideTab,
    tables: BTreeMap<String, Layout>,
    sorting: BTreeMap<String, Option<Sorting>>,
    views: BTreeMap<String, Mode>,
    window: Option<Frame>,
    system_theme: String,
}

impl Default for StateValues {
    fn default() -> Self {
        Self {
            volume: DEFAULT_VOLUME,
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            sidebar_open: true,
            sidebar_right_width: DEFAULT_SIDEBAR_RIGHT_WIDTH,
            sidebar_right_open: false,
            sidebar_right_tab: SideTab::Queue,
            tables: BTreeMap::new(),
            sorting: BTreeMap::new(),
            views: BTreeMap::new(),
            window: None,
            system_theme: ThemeKind::Dark.id().to_owned(),
        }
    }
}

#[derive(Clone)]
struct StateStore {
    path: PathBuf,
}

impl StateStore {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> Result<Option<StateValues>> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .context("cannot decode state"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                log::warn!("state: cannot read {}: {error}", self.path.display());
                Ok(None)
            }
        }
    }

    fn save(&self, state: &StateValues) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).context("cannot create state directory")?;
        }
        let bytes =
            serde_json::to_vec_pretty(state).context("cannot encode state")?;
        fs::write(&self.path, bytes).context("cannot write state")?;
        Ok(())
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "dark".to_owned(),
            adaptive_theme: true,
            icons: icons::BASE.to_owned(),
            rounding: Rounding::Rounded.id().to_owned(),
            blur: true,
            font_size: DEFAULT_FONT_SIZE,
            transparent: false,
            transparency: ui::BACKDROP_TRANSPARENCY,
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            server_side_decorations: true,
            #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
            window_rounding: Rounding::Square.id().to_owned(),
            window_controls: true,
            #[cfg(not(target_os = "macos"))]
            traffic_light_controls: false,
            controls_on_left: false,
            reduce_motion: Stillness::default().id().to_owned(),
            motion_pace: Pace::default().id().to_owned(),
            battery_saver: Saver::default().id().to_owned(),
            theme_overrides: ThemeOverrides::default(),
            fullscreen_controls_autohide: FullscreenControlsAutohide::Automatic.id().to_owned(),
        }
    }
}

pub struct AppSettings {
    values: Values,
    state: StateValues,
    path: PathBuf,
    store: StateStore,
    save: Option<Task<()>>,
    save_state: Option<Task<()>>,
    watch: Option<Subscription>,
    writable: bool,
}

impl AppSettings {
    pub fn load() -> Self {
        Self::load_from(settings_path(), state_path())
    }

    fn load_from(path: PathBuf, state_path: PathBuf) -> Self {
        let (bytes, writable) = match fs::read(&path) {
            Ok(bytes) => (Some(bytes), true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (None, true),
            Err(error) => {
                log::warn!("settings: cannot read {}: {error}", path.display());
                (None, false)
            }
        };
        let (values, writable) = match bytes.as_deref().map(serde_json::from_slice::<Values>) {
            Some(Ok(values)) => (values, writable),
            Some(Err(error)) => {
                log::warn!("settings: cannot parse {}: {error}", path.display());
                (Values::default(), false)
            }
            None => (Values::default(), writable),
        };

        let store = StateStore::new(state_path);
        let state = match store.load() {
            Ok(Some(saved)) => saved,
            Ok(None) => StateValues::default(),
            Err(error) => {
                log::warn!("settings: cannot load state: {error:#}");
                StateValues::default()
            }
        };

        Self {
            values,
            state,
            path,
            store,
            save: None,
            save_state: None,
            watch: None,
            writable,
        }
    }

    pub fn volume(&self) -> f32 {
        self.state.volume.clamp(0., 1.)
    }

    pub fn sidebar_width(&self) -> f32 {
        self.state.sidebar_width
    }

    pub fn sidebar_open(&self) -> bool {
        self.state.sidebar_open
    }

    pub fn sidebar_right_width(&self) -> f32 {
        self.state.sidebar_right_width
    }

    pub fn sidebar_right_open(&self) -> bool {
        self.state.sidebar_right_open
    }

    pub fn sidebar_right_tab(&self) -> SideTab {
        self.state.sidebar_right_tab
    }

    pub fn language(&self) -> &str {
        &self.values.language
    }

    pub fn font(&self) -> &str {
        &self.values.font
    }

    pub fn startup(&self) -> &str {
        &self.values.startup
    }

    pub fn theme(&self) -> &str {
        &self.values.appearance.theme
    }

    pub fn adaptive_theme(&self) -> bool {
        self.values.appearance.adaptive_theme
    }

    pub fn fullscreen_controls_autohide(&self) -> FullscreenControlsAutohide {
        FullscreenControlsAutohide::from_id(&self.values.appearance.fullscreen_controls_autohide)
    }

    pub fn icons(&self) -> &str {
        &self.values.appearance.icons
    }

    pub fn rounding(&self) -> &str {
        &self.values.appearance.rounding
    }

    pub fn blur(&self) -> bool {
        self.values.appearance.blur
    }

    pub fn stillness(&self) -> Stillness {
        Stillness::from_id(&self.values.appearance.reduce_motion)
    }

    pub fn pace(&self) -> Pace {
        Pace::from_id(&self.values.appearance.motion_pace)
    }

    pub fn saver(&self) -> Saver {
        Saver::from_id(&self.values.appearance.battery_saver)
    }

    pub fn system_theme(&self) -> ThemeKind {
        ThemeKind::from_id(&self.state.system_theme)
    }

    pub fn look(&self) -> Look {
        Look {
            kind: ThemeKind::from_id(self.theme()),
            rounding: Rounding::from_id(self.rounding()),
            font: self.font_size(),
            transparent: self.transparent(),
            transparency: self.transparency(),
            blur: self.blur(),
            tint: None,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn server_side_decorations(&self) -> bool {
        self.values.appearance.server_side_decorations
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn window_decorations(&self) -> WindowDecorations {
        match self.server_side_decorations() {
            true => WindowDecorations::Server,
            false => WindowDecorations::Client,
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    pub fn window_rounding(&self) -> Rounding {
        Rounding::from_id(&self.values.appearance.window_rounding)
    }

    pub fn window_controls(&self) -> bool {
        self.values.appearance.window_controls
    }

    #[cfg(not(target_os = "macos"))]
    pub fn traffic_light_controls(&self) -> bool {
        self.values.appearance.traffic_light_controls
    }

    pub fn controls_on_left(&self) -> bool {
        self.values.appearance.controls_on_left
    }

    pub fn font_size(&self) -> f32 {
        self.values
            .appearance
            .font_size
            .clamp(ui::MIN_FONT, ui::MAX_FONT)
    }

    pub fn transparent(&self) -> bool {
        self.values.appearance.transparent
    }

    pub fn transparency(&self) -> f32 {
        self.values
            .appearance
            .transparency
            .clamp(0., ui::MAX_TRANSPARENCY)
    }

    pub fn theme_overrides(&self) -> &ThemeOverrides {
        &self.values.appearance.theme_overrides
    }

    pub fn table(&self, table: &str) -> Layout {
        self.state.tables.get(table).cloned().unwrap_or_default()
    }

    pub fn sorting(&self, table: &str) -> Option<Option<Sorting>> {
        self.state.sorting.get(table).cloned()
    }

    pub fn view_or(&self, table: &str, fallback: Mode) -> Mode {
        self.state.views.get(table).copied().unwrap_or(fallback)
    }

    pub fn nav_shown(&self, entry: &str) -> bool {
        !self.values.hidden_nav.contains_key(entry)
    }

    pub fn ensure_file(&self) -> PathBuf {
        if !self.path.exists() {
            self.save_now();
        }
        self.path.clone()
    }

    pub fn set_volume(&mut self, volume: f32, cx: &mut Context<Self>) {
        self.state.volume = volume.clamp(0., 1.);
        self.schedule_state_save(cx);
    }

    pub fn set_language(&mut self, language: impl Into<String>, cx: &mut Context<Self>) {
        self.values.language = language.into();
        i18n::set(i18n::resolve(&self.values.language));
        cx.refresh_windows();
        self.schedule_save(cx);
    }

    pub fn set_font(&mut self, font: impl Into<String>, cx: &mut Context<Self>) {
        let font = font.into();
        if self.values.font == font {
            return;
        }
        self.values.font = font;
        cx.refresh_windows();
        self.schedule_save(cx);
    }

    pub fn set_sidebar(&mut self, width: f32, open: bool, cx: &mut Context<Self>) {
        self.state.sidebar_width = width;
        self.state.sidebar_open = open;
        self.schedule_state_save(cx);
    }

    pub fn set_sidebar_right_width(&mut self, width: f32, cx: &mut Context<Self>) {
        self.state.sidebar_right_width = width;
        self.schedule_state_save(cx);
    }

    pub fn set_sidebar_right_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.state.sidebar_right_open = open;
        self.schedule_state_save(cx);
    }

    pub fn set_sidebar_right_tab(&mut self, tab: SideTab, cx: &mut Context<Self>) {
        if self.state.sidebar_right_tab == tab {
            return;
        }
        self.state.sidebar_right_tab = tab;
        self.schedule_state_save(cx);
    }

    pub fn set_startup(&mut self, screen: impl Into<String>, cx: &mut Context<Self>) {
        let screen = screen.into();
        if self.values.startup == screen {
            return;
        }
        self.values.startup = screen;
        self.schedule_save(cx);
    }

    pub fn set_theme(&mut self, theme: impl Into<String>, cx: &mut Context<Self>) {
        self.values.appearance.theme = theme.into();
        self.schedule_save(cx);
    }

    pub fn set_adaptive_theme(&mut self, adaptive: bool, cx: &mut Context<Self>) {
        self.values.appearance.adaptive_theme = adaptive;
        self.schedule_save(cx);
    }

    pub fn set_icons(&mut self, pack: impl Into<String>, cx: &mut Context<Self>) {
        let pack = pack.into();
        if self.values.appearance.icons == pack {
            return;
        }
        icons::set(&pack);
        self.values.appearance.icons = pack;
        cx.refresh_windows();
        self.schedule_save(cx);
    }

    pub fn set_rounding(&mut self, rounding: impl Into<String>, cx: &mut Context<Self>) {
        self.values.appearance.rounding = rounding.into();
        self.schedule_save(cx);
    }

    pub fn set_blur(&mut self, blur: bool, cx: &mut Context<Self>) {
        self.values.appearance.blur = blur;
        self.schedule_save(cx);
    }

    pub fn set_stillness(&mut self, stillness: Stillness, cx: &mut Context<Self>) {
        if self.stillness() == stillness {
            return;
        }
        self.values.appearance.reduce_motion = stillness.id().to_owned();
        ui::motion::apply(stillness, self.pace(), cx);
        self.schedule_save(cx);
    }

    pub fn set_pace(&mut self, pace: Pace, cx: &mut Context<Self>) {
        if self.pace() == pace {
            return;
        }
        self.values.appearance.motion_pace = pace.id().to_owned();
        ui::motion::apply(self.stillness(), pace, cx);
        self.schedule_save(cx);
    }

    pub fn set_system_theme(&mut self, kind: ThemeKind, cx: &mut Context<Self>) {
        if self.system_theme() == kind {
            return;
        }
        self.state.system_theme = kind.id().to_owned();
        self.schedule_state_save(cx);
    }

    pub fn set_saver(&mut self, saver: Saver, cx: &mut Context<Self>) {
        if self.saver() == saver {
            return;
        }
        self.values.appearance.battery_saver = saver.id().to_owned();
        self.schedule_save(cx);
    }

    pub fn set_table(&mut self, table: &str, layout: Layout, cx: &mut Context<Self>) {
        if self.state.tables.get(table) == Some(&layout) {
            return;
        }
        self.state.tables.insert(table.to_owned(), layout);
        self.schedule_state_save(cx);
    }

    pub fn set_view(&mut self, table: &str, mode: Mode, cx: &mut Context<Self>) {
        if self.state.views.get(table) == Some(&mode) {
            return;
        }
        self.state.views.insert(table.to_owned(), mode);
        self.schedule_state_save(cx);
    }

    pub fn set_sorting(&mut self, table: &str, sorting: Option<Sorting>, cx: &mut Context<Self>) {
        if self.state.sorting.get(table) == Some(&sorting) {
            return;
        }
        self.state.sorting.insert(table.to_owned(), sorting);
        self.schedule_state_save(cx);
    }

    pub fn set_nav_shown(&mut self, entry: &str, shown: bool, cx: &mut Context<Self>) {
        if self.nav_shown(entry) == shown {
            return;
        }
        match shown {
            true => self.values.hidden_nav.remove(entry),
            false => self.values.hidden_nav.insert(entry.to_owned(), true),
        };
        self.schedule_save(cx);
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn set_server_side_decorations(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.values.appearance.server_side_decorations = shown;
        self.schedule_save(cx);
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    pub fn set_window_rounding(&mut self, rounding: Rounding, cx: &mut Context<Self>) {
        self.values.appearance.window_rounding = rounding.id().to_owned();
        self.schedule_save(cx);
    }

    pub fn set_fullscreen_controls_autohide(
        &mut self,
        fca: FullscreenControlsAutohide,
        cx: &mut Context<Self>,
    ) {
        self.values.appearance.fullscreen_controls_autohide = fca.id().to_owned();
        self.schedule_save(cx);
    }

    pub fn set_window_controls(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.values.appearance.window_controls = shown;
        self.schedule_save(cx);
    }

    #[cfg(not(target_os = "macos"))]
    pub fn set_traffic_light_controls(&mut self, traffic_light: bool, cx: &mut Context<Self>) {
        self.values.appearance.traffic_light_controls = traffic_light;
        self.schedule_save(cx);
    }

    pub fn set_controls_on_left(&mut self, left: bool, cx: &mut Context<Self>) {
        self.values.appearance.controls_on_left = left;
        self.schedule_save(cx);
    }

    pub fn set_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        self.values.appearance.font_size = size.clamp(ui::MIN_FONT, ui::MAX_FONT);
        self.schedule_save(cx);
    }

    pub fn set_transparent(&mut self, transparent: bool, cx: &mut Context<Self>) {
        self.values.appearance.transparent = transparent;
        self.schedule_save(cx);
    }

    pub fn set_transparency(&mut self, transparency: f32, cx: &mut Context<Self>) {
        self.values.appearance.transparency = transparency.clamp(0., ui::MAX_TRANSPARENCY);
        self.schedule_save(cx);
    }

    pub fn watch_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.keep_frame(window, cx);
        self.watch = Some(cx.observe_window_bounds(window, |this, window, cx| {
            this.keep_frame(window, cx);
        }));
    }

    fn keep_frame(&mut self, window: &Window, cx: &mut Context<Self>) {
        let frame = Frame::of(window);
        if !frame.sane() || self.state.window == Some(frame) {
            return;
        }
        self.state.window = Some(frame);
        self.schedule_state_save(cx);
    }

    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        cx.notify();
        self.save_quietly(cx);
    }

    fn save_quietly(&mut self, cx: &mut Context<Self>) {
        self.save = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            this.update(cx, |this, _| this.save_now()).ok();
        }));
    }

    fn schedule_state_save(&mut self, cx: &mut Context<Self>) {
        cx.notify();
        self.save_state_quietly(cx);
    }

    fn save_state_quietly(&mut self, cx: &mut Context<Self>) {
        self.save_state = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            this.update(cx, |this, _| this.save_state_now()).ok();
        }));
    }

    fn save_state_now(&self) {
        if let Err(error) = self.store.save(&self.state) {
            log::error!("settings: cannot save state: {error:#}");
        }
    }

    fn save_now(&self) -> bool {
        if !self.writable {
            return false;
        }
        let Some(parent) = self.path.parent() else {
            return false;
        };
        if let Err(error) = fs::create_dir_all(parent) {
            log::error!("settings: cannot create {}: {error}", parent.display());
            return false;
        }

        let bytes = match serde_json::to_vec_pretty(&self.values) {
            Ok(bytes) => bytes,
            Err(error) => {
                log::error!("settings: cannot serialize values: {error}");
                return false;
            }
        };
        if let Err(error) = fs::write(&self.path, bytes) {
            log::error!("settings: cannot write {}: {error}", self.path.display());
            return false;
        }
        true
    }
}

pub fn window_placement(least: Size<Pixels>, cx: &App) -> Option<(WindowBounds, DisplayId)> {
    let frame = Sonora::global(cx).settings.read(cx).state.window?;
    if !frame.sane() {
        return None;
    }

    let placement = frame.placement(least);
    let bounds = placement.get_bounds();
    cx.displays()
        .iter()
        .find(|display| display.bounds().contains(&bounds.center()))
        .map(|display| (placement, display.id()))
}

pub fn remember_window(window: &mut Window, cx: &mut App) {
    let settings = Sonora::global(cx).settings.clone();
    settings.update(cx, |settings, cx| settings.watch_window(window, cx));
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sonora")
        .join("settings.json")
}

fn state_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sonora")
        .join("state.json")
}