use ui::{
    ActiveTheme as _, Button, Panel, Scroller, Side, SNUG,
};

use gpui::prelude::*;
use gpui::{
    App, AnyElement, Context, ElementId, Entity, Pixels, Render, ScrollHandle, Window, div, px,
};
use router::{
    Destination, MediatekTab, Navigation, NavigationEvent, SettingsTab, navigate,
};

const NAV: [(Option<router::NavEntry>, &str, Destination); 7] = [
    (
        Some(router::NavEntry::Mediatek),
        "icons/folder.svg",
        Destination::Mediatek(MediatekTab::Flasher),
    ),
    (
        Some(router::NavEntry::Unisoc),
        "icons/chip.svg",
        Destination::Unisoc,
    ),
    (
        Some(router::NavEntry::Xiaomi),
        "icons/smartphone.svg",
        Destination::Xiaomi,
    ),
    (
        Some(router::NavEntry::Firmwares),
        "icons/package.svg",
        Destination::Firmwares,
    ),
    (
        Some(router::NavEntry::Terminal),
        "icons/terminal.svg",
        Destination::Terminal,
    ),
    (
        Some(router::NavEntry::Drivers),
        "icons/usb.svg",
        Destination::Drivers,
    ),
    (
        None,
        "icons/settings.svg",
        Destination::Settings(SettingsTab::General),
    ),
];

const MEDIATEK_TABS: [(&str, MediatekTab); 4] = [
    ("nav-mediatek-flasher", MediatekTab::Flasher),
    ("nav-mediatek-partitions", MediatekTab::Partitions),
    ("nav-mediatek-reboot-power", MediatekTab::RebootPower),
    ("nav-mediatek-settings", MediatekTab::Settings),
];

const MIN_WIDTH: Pixels = px(160.);
const MAX_WIDTH: Pixels = px(400.);

#[derive(Clone, Copy, PartialEq)]
enum Group {
    Mediatek,
    Settings,
}

impl Group {
    fn of(destination: &Destination) -> Option<Self> {
        match destination {
            Destination::Mediatek(_) => Some(Self::Mediatek),
            Destination::Settings(_) => Some(Self::Settings),
            _ => None,
        }
    }
}

pub(crate) struct SidebarLeft {
    trail: Entity<Navigation>,
    at: Destination,
    width: Pixels,
    open: bool,
    cramped: bool,
    forced: Option<bool>,
    mediatek_open: bool,
    settings_open: bool,
    scrollbar: Entity<ui::Scrollbar>,
}

impl SidebarLeft {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let trail = router::trail(cx);

        cx.observe(&trail, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&trail, |_this, _, _: &NavigationEvent, _cx| {
        })
        .detach();

        let at = trail.read(cx).current();
        let mediatek_open = matches!(at, Destination::Mediatek(_));
        let settings_open = matches!(at, Destination::Settings(_));

        let width = px(200.).clamp(MIN_WIDTH, MAX_WIDTH);
        let open = true;

        let entity_id = cx.entity_id();
        Self {
            trail,
            at,
            width,
            open,
            forced: None,
            cramped: false,
            mediatek_open,
            settings_open,
            scrollbar: cx.new(move |_| ui::Scrollbar::new(ScrollHandle::new()).watching(entity_id)),
        }
    }

    fn follow(&mut self, current: &Destination) {
        if self.at == *current {
            return;
        }
        self.at = current.clone();
        let (mediatek, settings) = expanded(current);
        self.mediatek_open |= mediatek;
        self.settings_open |= settings;
    }

    pub fn is_open(&self) -> bool {
        self.forced.unwrap_or(self.open && !self.cramped)
    }

    pub fn overlays(&self) -> bool {
        self.cramped && self.is_open()
    }

    pub fn overlay_width(&self) -> Pixels {
        match self.overlays() {
            true => self.width,
            false => Pixels::ZERO,
        }
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.overlays() {
            return;
        }
        self.forced = Some(false);
        cx.notify();
    }

    pub fn occupied_width(&self) -> Pixels {
        match self.is_open() && !self.overlays() {
            true => self.width,
            false => Pixels::ZERO,
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        match self.cramped {
            true => self.forced = Some(!self.is_open()),
            false => {
                self.open = !self.open;
            }
        }
        cx.notify();
    }

    fn ceiling(&self, window: &Window, cx: &Context<Self>) -> Pixels {
        let reserved = match self.overlays() {
            true => Pixels::ZERO,
            false => SNUG + super::Chrome::sidebar_right(cx),
        };

        super::cap(MIN_WIDTH, MAX_WIDTH, reserved, window)
    }

    pub fn adapt(&mut self, right: Pixels, window: &Window, cx: &mut App) {
        self.width = ui::snapped(self.width, window);

        let space_left = window.viewport_size().width - self.width - right;
        let cramped = space_left < SNUG;
        if cramped != self.cramped {
            self.cramped = cramped;
            self.forced = None;
            cx.refresh_windows();
        }
    }

    fn navigation(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut rows = Vec::new();
        for (index, (_, _, destination)) in NAV.iter().enumerate() {
            let group = Group::of(destination);
            rows.push(self.nav(index, cx));
            if let Some(group) = group.filter(|group| self.opened(*group)) {
                rows.push(self.tabs(group, cx));
            }
        }
        rows
    }

    fn opened(&self, group: Group) -> bool {
        match group {
            Group::Mediatek => self.mediatek_open,
            Group::Settings => self.settings_open,
        }
    }

    fn flip(&mut self, group: Group) {
        let open = match group {
            Group::Mediatek => &mut self.mediatek_open,
            Group::Settings => &mut self.settings_open,
        };
        *open = !*open;
    }

    fn nav(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let accent = theme.sidebar_accent;
        let (_, icon, destination) = NAV[index].clone();
        let current = self.trail.read(cx).current();
        let group = Group::of(&destination);
        let active = match group {
            Some(group) => Group::of(&current) == Some(group),
            None => destination.same_section(&current),
        };
        let tint = match active {
            true => theme.foreground,
            false => theme.muted_foreground,
        };

        let row = Button::new(index.into())
            .ghost()
            .icon(icon)
            .tint(tint)
            .gap_2p5()
            .justify_start()
            .hover(move |style| style.bg(accent))
            .active(move |style| style.bg(accent));

        match group {
            Some(group) => row
                .trailing(chevron(self.opened(group)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.flip(group);
                    cx.notify();
                })),
            None => row
                .when(active, |button| button.bg(accent))
                .on_click(move |_, _, cx| navigate(destination.clone(), cx)),
        }
        .into_any_element()
    }

    fn tabs(&self, group: Group, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let accent = theme.sidebar_accent;
        let current = self.trail.read(cx).current();

        let tab = |id: ElementId, destination: Destination| {
            let chosen = destination == current;
            let tint = match chosen {
                true => theme.foreground,
                false => theme.muted_foreground,
            };

            Button::new(id)
                .ghost()
                .tint(tint)
                .gap_2p5()
                .justify_start()
                .hover(move |style| style.bg(accent))
                .active(move |style| style.bg(accent))
                .when(chosen, |button| button.bg(accent))
                .on_click(move |_, _, cx| navigate(destination.clone(), cx))
        };

        match group {
            Group::Mediatek => {
                let mut items: Vec<AnyElement> = MEDIATEK_TABS
                    .into_iter()
                    .map(|(name, tab_id)| {
                        tab(name.into(), Destination::Mediatek(tab_id))
                            .label(i18n::lookup(name, None))
                            .into_any_element()
                    })
                    .collect();
                div().flex().flex_col().gap_1().children(items).into_any_element()
            }
            Group::Settings => {
                tab("settings-tab-general".into(), Destination::Settings(SettingsTab::General))
                    .label(i18n::lookup("settings-tab-general", None))
                    .into_any_element()
            }
        }
    }
}

impl Render for SidebarLeft {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let sidebar_bg = theme.sidebar;
        let sidebar_border = theme.sidebar_border;

        let current = self.trail.read(cx).current();
        self.follow(&current);
        self.adapt(super::Chrome::sidebar_right(cx), window, cx);

        let mut rows = self.navigation(cx);

        let overlaid = self.overlays();
        let panel = Panel::new("sidebar-left", Side::Left, self.width)
            .limits(MIN_WIDTH, MAX_WIDTH)
            .reach(self.ceiling(window, cx))
            .clears_scrollbar()
            .on_resize(cx.listener(|this, width: &Pixels, _, cx| {
                this.width = *width;
                cx.notify();
            }))
            .when(!self.is_open(), |this| this.hidden())
            .when(!theme.transparent, |this| this.bg(sidebar_bg))
            .border_color(sidebar_border)
            .when(overlaid, |this| {
                this.occlude().absolute().left_0().top_0().bottom_0()
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(
                        Scroller::new("sidebar-left-rows", &self.scrollbar)
                            .size_full()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .w_full()
                                    .p_3()
                                    .pb(ui::perch_room(cx))
                                    .children(rows),
                            ),
                    )
                    .children(ui::return_top("sidebar-return-top", &self.scrollbar, cx)),
            );

        match overlaid {
            false => panel.into_any_element(),
            true => div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .child(
                    ui::Shield::new("sidebar-shield")
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _: &gpui::MouseDownEvent, _, cx| this.dismiss(cx)),
                        ),
                )
                .child(panel)
                .into_any_element(),
        }
    }
}

fn expanded(current: &Destination) -> (bool, bool) {
    (
        matches!(current, Destination::Mediatek(_)),
        matches!(current, Destination::Settings(_)),
    )
}

fn chevron(open: bool) -> &'static str {
    match open {
        true => "icons/chevron-down.svg",
        false => "icons/chevron-right.svg",
    }
}