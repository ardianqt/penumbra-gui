mod link;
mod navigation;
mod uri;

pub use link::Link;
pub use navigation::{Navigation, NavigationEvent};
pub use uri::destination;

use gpui::{App, AppContext as _, Entity, Global};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediatekTab {
    Flasher,
    Partitions,
    RebootPower,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavEntry {
    Mediatek,
    Unisoc,
    Xiaomi,
    Firmwares,
    Terminal,
    Drivers,
}

impl NavEntry {
    pub const ALL: [Self; 6] = [
        Self::Mediatek,
        Self::Unisoc,
        Self::Xiaomi,
        Self::Firmwares,
        Self::Terminal,
        Self::Drivers,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Mediatek => "mediatek",
            Self::Unisoc => "unisoc",
            Self::Xiaomi => "xiaomi",
            Self::Firmwares => "firmwares",
            Self::Terminal => "terminal",
            Self::Drivers => "drivers",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Mediatek => "nav-mediatek",
            Self::Unisoc => "nav-unisoc",
            Self::Xiaomi => "nav-xiaomi",
            Self::Firmwares => "nav-firmwares",
            Self::Terminal => "nav-terminal",
            Self::Drivers => "nav-drivers",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    MediatekFlasher,
    MediatekPartitions,
    MediatekRebootPower,
    MediatekSettings,
    Unisoc,
    Xiaomi,
    Firmwares,
    Terminal,
    Drivers,
}

impl Screen {
    pub const ALL: [Self; 9] = [
        Self::MediatekFlasher,
        Self::MediatekPartitions,
        Self::MediatekRebootPower,
        Self::MediatekSettings,
        Self::Unisoc,
        Self::Xiaomi,
        Self::Firmwares,
        Self::Terminal,
        Self::Drivers,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::MediatekFlasher => "mediatek-flasher",
            Self::MediatekPartitions => "mediatek-partitions",
            Self::MediatekRebootPower => "mediatek-reboot-power",
            Self::MediatekSettings => "mediatek-settings",
            Self::Unisoc => "unisoc",
            Self::Xiaomi => "xiaomi",
            Self::Firmwares => "firmwares",
            Self::Terminal => "terminal",
            Self::Drivers => "drivers",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::MediatekFlasher => "nav-mediatek-flasher",
            Self::MediatekPartitions => "nav-mediatek-partitions",
            Self::MediatekRebootPower => "nav-mediatek-reboot-power",
            Self::MediatekSettings => "nav-mediatek-settings",
            Self::Unisoc => "nav-unisoc",
            Self::Xiaomi => "nav-xiaomi",
            Self::Firmwares => "nav-firmwares",
            Self::Terminal => "nav-terminal",
            Self::Drivers => "nav-drivers",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|screen| screen.id() == id)
    }

    pub fn destination(self) -> Destination {
        match self {
            Self::MediatekFlasher => Destination::Mediatek(MediatekTab::Flasher),
            Self::MediatekPartitions => Destination::Mediatek(MediatekTab::Partitions),
            Self::MediatekRebootPower => Destination::Mediatek(MediatekTab::RebootPower),
            Self::MediatekSettings => Destination::Mediatek(MediatekTab::Settings),
            Self::Unisoc => Destination::Unisoc,
            Self::Xiaomi => Destination::Xiaomi,
            Self::Firmwares => Destination::Firmwares,
            Self::Terminal => Destination::Terminal,
            Self::Drivers => Destination::Drivers,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    General,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    Mediatek(MediatekTab),
    Unisoc,
    Xiaomi,
    Firmwares,
    Terminal,
    Drivers,
    Settings(SettingsTab),
}

impl Destination {
    pub fn same_section(&self, other: &Destination) -> bool {
        match (self, other) {
            (Destination::Mediatek(_), Destination::Mediatek(_))
            | (Destination::Settings(_), Destination::Settings(_)) => true,
            _ => self == other,
        }
    }
}

#[derive(Clone)]
struct Router(Entity<Navigation>);

impl Global for Router {}

pub fn init(start: Destination, cx: &mut App) {
    let navigation = cx.new(|_| Navigation::new(start));
    cx.set_global(Router(navigation));
}

pub fn trail(cx: &App) -> Entity<Navigation> {
    cx.global::<Router>().0.clone()
}

pub fn navigate(destination: Destination, cx: &mut App) {
    trail(cx).update(cx, |navigation, cx| navigation.go(destination, cx));
}

pub fn back(cx: &mut App) {
    trail(cx).update(cx, |navigation, cx| navigation.back(cx));
}

pub fn forward(cx: &mut App) {
    trail(cx).update(cx, |navigation, cx| navigation.forward(cx));
}