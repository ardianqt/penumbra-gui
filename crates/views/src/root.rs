use gpui::{AnyView, Context, Entity, Render};
use gpui::{App, Font, SharedString, font, prelude::*};
use gpui::{Window, div};
use router::{Destination, MediatekTab, NavigationEvent, SettingsTab, back, navigate};
use state::{AppSettings, Sonora};
use ui::{ActiveTheme as _, Theme, ThemeKind, Stillness, Look};

use crate::chrome::{TitleBar, TitleBarEvent, TitleBarOptions};
use crate::shells::workspace::Workspace;
use crate::{MediatekView, SettingsView};

struct Screens {
    settings: Entity<SettingsView>,
    mediatek: Entity<MediatekView>,
}

struct Shells {
    workspace: Entity<Workspace>,
}

pub struct Root {
    title_bar: Entity<TitleBar>,
    shells: Shells,
    screens: Screens,
    _adaptive: Entity<Adaptive>,
}

struct Adaptive;

impl Render for Adaptive {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl Root {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let navigation = router::trail(cx);

        cx.subscribe(&navigation, |this, _, event, cx| {
            let NavigationEvent::Moved(destination) = event;
            this.transition_to(destination.clone(), cx);
        })
        .detach();

        cx.observe_window_activation(window, |_, window, cx| {
            if !window.is_window_active() {
                return;
            }
            let settings = Sonora::global(cx).settings.clone();
            let (stillness, pace) = {
                let settings = settings.read(cx);
                (settings.stillness(), settings.pace())
            };
            if stillness != Stillness::System {
                return;
            }
            ui::motion::apply(stillness, pace, cx);
        })
        .detach();

        window
            .observe_window_appearance(|_, cx| {
                let settings = Sonora::global(cx).settings.read(cx);
                let reported = ThemeKind::reported(cx);
                let changed = ThemeKind::assumed() != Some(reported);
                if changed {
                    ThemeKind::assume(reported);
                }
                if ThemeKind::from_id(settings.theme()) != ThemeKind::System || !changed {
                    return;
                }
                let settings = Sonora::global(cx).settings.read(cx);
                let look = Look {
                    tint: cx.theme().tint,
                    ..settings.look()
                };
                let overrides = settings.theme_overrides().clone();
                Theme::fade(look, &overrides, cx);
            })
            .detach();

        let mediatek = cx.new(|_| MediatekView::new(MediatekTab::Flasher, cx));
        let settings = cx.new(|_| SettingsView::new(cx));

        let start = navigation.read(cx).current();
        let workspace = cx.new(|_| Workspace::new(cx));

        let title_bar = cx.new(TitleBar::new);
        cx.subscribe(&title_bar, |this, _, event, cx| match event {
            TitleBarEvent::ToggleSidebar => this
                .shells
                .workspace
                .update(cx, |workspace, cx| workspace.toggle_sidebar(cx)),
            TitleBarEvent::ToggleSidebarRight => this
                .shells
                .workspace
                .update(cx, |workspace, cx| workspace.toggle_sidebar_right(cx)),
        })
        .detach();

        let adaptive = cx.new(|_| Adaptive);

        let mut root = Self {
            title_bar,
            shells: Shells { workspace },
            screens: Screens { settings, mediatek },
            _adaptive: adaptive,
        };
        root.show(start, cx);
        root
    }

    fn transition_to(&mut self, destination: Destination, cx: &mut Context<Self>) {
        self.show(destination, cx);
    }

    fn show(&mut self, destination: Destination, cx: &mut Context<Self>) {
        let content: AnyView = match &destination {
            Destination::Mediatek(tab) => {
                self.screens.mediatek.update(cx, |view, cx| view.select(*tab, cx));
                self.screens.mediatek.clone().into()
            }
            Destination::Unisoc => {
                crate::screens::unisoc::UnisocView.into()
            }
            Destination::Xiaomi => {
                crate::screens::xiaomi::XiaomiView.into()
            }
            Destination::Firmwares => {
                crate::screens::firmwares::FirmwaresView.into()
            }
            Destination::Terminal => {
                crate::screens::terminal::TerminalView.into()
            }
            Destination::Drivers => {
                crate::screens::drivers::DriversView.into()
            }
            Destination::Settings(_tab) => {
                self.screens.settings.clone().into()
            }
        };

        self.shells
            .workspace
            .update(cx, |workspace, cx| workspace.set_content(content, cx));
        cx.notify();
    }
}

const UI_FONT: &str = "Inter";

const SCRIPTS: [&str; 18] = [
    "Source Han Sans",
    "Noto Sans CJK JP",
    "Noto Sans CJK SC",
    "Noto Sans CJK TC",
    "Noto Sans CJK KR",
    "Noto Sans Arabic",
    "Noto Sans Hebrew",
    "Noto Sans Thai",
    "Noto Sans Devanagari",
    "Hiragino Sans",
    "PingFang SC",
    "Apple SD Gothic Neo",
    "Yu Gothic UI",
    "Microsoft YaHei UI",
    "Malgun Gothic",
    "Noto Color Emoji",
    "Apple Color Emoji",
    "Segoe UI Emoji",
];

fn ui_font(cx: &App) -> Font {
    let chosen = Sonora::global(cx).settings.read(cx).font();
    match chosen == state::SYSTEM_FONT {
        true => Font {
            fallbacks: Some(scripts(false).clone()),
            ..font(UI_FONT)
        },
        false => Font {
            fallbacks: Some(scripts(true).clone()),
            ..font(SharedString::from(chosen.to_owned()))
        },
    }
}

fn scripts(custom: bool) -> &'static gpui::FontFallbacks {
    static BUNDLED: std::sync::OnceLock<gpui::FontFallbacks> = std::sync::OnceLock::new();
    static CHOSEN: std::sync::OnceLock<gpui::FontFallbacks> = std::sync::OnceLock::new();
    let named = || SCRIPTS.iter().map(|name| (*name).to_owned());
    match custom {
        true => CHOSEN.get_or_init(|| {
            gpui::FontFallbacks::from_fonts(std::iter::once(UI_FONT.to_owned()).chain(named()).collect())
        }),
        false => BUNDLED.get_or_init(|| gpui::FontFallbacks::from_fonts(named().collect())),
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        window.set_rem_size(theme.font_size);

        div()
            .relative()
            .flex()
            .font(ui_font(cx))
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.title_bar.clone())
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.shells.workspace.clone()),
            )
    }
}