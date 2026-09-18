pub(crate) mod flasher;
pub(crate) mod partitions;
pub(crate) mod reboot_power;
pub(crate) mod settings;

use gpui::prelude::*;
use gpui::{Context, Entity, Render, Window};
use router::MediatekTab;

use flasher::MediatekFlasher;
use partitions::MediatekPartitions;
use reboot_power::MediatekRebootPower;
use settings::MediatekSettings;

pub(crate) struct MediatekView {
    tab: MediatekTab,
    flasher: Entity<MediatekFlasher>,
    partitions: Entity<MediatekPartitions>,
    reboot_power: Entity<MediatekRebootPower>,
    mediatek_settings: Entity<MediatekSettings>,
}

impl MediatekView {
    pub fn new(tab: MediatekTab, cx: &mut Context<Self>) -> Self {
        Self {
            tab,
            flasher: cx.new(MediatekFlasher::new),
            partitions: cx.new(MediatekPartitions::new),
            reboot_power: cx.new(MediatekRebootPower::new),
            mediatek_settings: cx.new(MediatekSettings::new),
        }
    }

    pub fn select(&mut self, tab: MediatekTab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }
}

impl Render for MediatekView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.tab {
            MediatekTab::Flasher => self.flasher.clone().into_any_element(),
            MediatekTab::Partitions => self.partitions.clone().into_any_element(),
            MediatekTab::RebootPower => self.reboot_power.clone().into_any_element(),
            MediatekTab::Settings => self.mediatek_settings.clone().into_any_element(),
        }
    }
}