/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2026 Shomy
*/

use std::sync::{Arc, Mutex};

use acon::{MMIO, SoC};
use hacc::BootControl;

use crate::Partition;

#[derive(Clone, Default)]
pub struct DevInfo {
    inner: Arc<Mutex<DevInfoData>>,
}

#[derive(Default, Clone)]
pub struct DevInfoData {
    pub soc_id: [u8; 32],
    pub meid: [u8; 16],
    pub hw_code: u16,
    pub hw_subcode: u16,
    pub chip: Option<SoC>,
    pub partitions: Vec<Partition>,
    pub bootctrl: Option<BootControl>,
    pub target_config: u32,
}

impl DevInfo {
    pub fn new(data: DevInfoData) -> Self {
        Self { inner: Arc::new(Mutex::new(data)) }
    }

    pub fn data(&self) -> DevInfoData {
        self.inner.lock().unwrap().clone()
    }

    pub fn set_data(&self, data: DevInfoData) {
        *self.inner.lock().unwrap() = data;
    }

    pub fn soc_id(&self) -> [u8; 32] {
        self.inner.lock().unwrap().soc_id
    }

    pub fn set_soc_id(&self, soc_id: [u8; 32]) {
        self.inner.lock().unwrap().soc_id = soc_id;
    }

    pub fn meid(&self) -> [u8; 16] {
        self.inner.lock().unwrap().meid
    }

    pub fn set_meid(&self, meid: [u8; 16]) {
        self.inner.lock().unwrap().meid = meid;
    }

    pub fn hw_subcode(&self) -> u16 {
        self.inner.lock().unwrap().hw_subcode
    }

    pub fn set_hw_subcode(&self, hw_subcode: u16) {
        self.inner.lock().unwrap().hw_subcode = hw_subcode;
    }

    pub fn chip(&self) -> Option<SoC> {
        self.inner.lock().unwrap().chip
    }

    pub fn set_chip(&self, chip: SoC) {
        self.inner.lock().unwrap().chip = Some(chip);
    }

    pub fn clear_chip(&self) {
        self.inner.lock().unwrap().chip = None;
    }

    pub fn hw_code(&self) -> u16 {
        self.chip().map(|c| c.to_hwcode()).unwrap_or(self.inner.lock().unwrap().hw_code)
    }

    pub fn partitions(&self) -> Vec<Partition> {
        self.inner.lock().unwrap().partitions.clone()
    }

    pub fn set_partitions(&self, partitions: Vec<Partition>) {
        self.inner.lock().unwrap().partitions = partitions;
    }

    pub fn get_partition(&self, name: &str) -> Option<Partition> {
        let data = self.inner.lock().unwrap();

        if let Some(p) = data.partitions.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
            return Some(p.clone());
        }

        let suffix = data.bootctrl.as_ref()?.get_current_suffix().map(|s| s.to_string())?;

        let suffixed_name = format!("{name}{suffix}");

        data.partitions.iter().find(|p| p.name.eq_ignore_ascii_case(&suffixed_name)).cloned()
    }

    pub fn bootctrl(&self) -> Option<BootControl> {
        self.inner.lock().unwrap().bootctrl.clone()
    }

    pub fn set_bootctrl(&self, bootctrl: BootControl) {
        self.inner.lock().unwrap().bootctrl = Some(bootctrl);
    }

    pub fn target_config(&self) -> u32 {
        self.inner.lock().unwrap().target_config
    }

    pub fn set_target_config(&self, cfg: u32) {
        self.inner.lock().unwrap().target_config = cfg;
    }

    pub fn sbc_enabled(&self) -> bool {
        (self.target_config() & 0x1) != 0
    }

    pub fn sla_enabled(&self) -> bool {
        (self.target_config() & 0x2) != 0
    }

    pub fn daa_enabled(&self) -> bool {
        (self.target_config() & 0x4) != 0
    }
}