use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use penumbra_mtk::da::BootMode;
use penumbra_mtk::device::{Device, DeviceBuilder};
use penumbra_mtk::port::{PortBackend, PortType};

#[derive(Clone)]
pub struct DeviceInfo {
    pub chip_name: String,
    pub hw_code: u32,
    pub storage_type: String,
    pub sbc: bool,
    pub sla: bool,
    pub daa: bool,
}

pub struct MtkConnection {
    cancel: Arc<AtomicBool>,
}

impl MtkConnection {
    pub fn new() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub fn reset_cancel(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn connect(da_bytes: &[u8]) -> Result<(Device, DeviceInfo), String> {
        let port = PortType::find_and_open(None, None, PortBackend::Auto)
            .map_err(|e| format!("Port error: {e}"))?
            .ok_or_else(|| "No MediaTek device found".to_string())?;

        let mut device = DeviceBuilder::new(port)
            .with_da_data(da_bytes)
            .build()
            .map_err(|e| format!("Device build error: {e}"))?;

        device.init().map_err(|e| format!("Init failed: {e}"))?;

        let devinfo = device.devinfo();
        let chip_name = devinfo.chip().map(|c| format!("{:?}", c)).unwrap_or_else(|| "Unknown".into());

        let info = DeviceInfo {
            chip_name: chip_name.to_string(),
            hw_code: devinfo.hw_code() as u32,
            storage_type: "Unknown".into(),
            sbc: devinfo.sbc_enabled(),
            sla: devinfo.sla_enabled(),
            daa: devinfo.daa_enabled(),
        };

        Ok((device, info))
    }

    pub fn reboot(device: &mut Device, mode: &str) -> Result<(), String> {
        let boot_mode = match mode {
            "Normal Boot" => BootMode::Normal,
            "Fastboot" => BootMode::Fastboot,
            "Meta Mode" => BootMode::Meta,
            "Power Off" => BootMode::Off,
            _ => return Err(format!("Unknown reboot mode: {mode}")),
        };
        device.reboot(boot_mode).map_err(|e| format!("Reboot failed: {e}"))
    }

    pub fn read_partition(
        device: &mut Device,
        partition: &str,
        output_path: &Path,
    ) -> Result<String, String> {
        let file = std::fs::File::create(output_path)
            .map_err(|e| format!("Cannot create output file: {e}"))?;
        let mut writer = std::io::BufWriter::new(file);
        device.read_partition(partition, &mut writer, |_, _| {})
            .map_err(|e| format!("Read failed: {e}"))?;
        Ok(format!("Saved {partition} to {}", output_path.display()))
    }

    pub fn erase_partition(device: &mut Device, partition: &str) -> Result<String, String> {
        device.erase_partition(partition, |_, _| {})
            .map_err(|e| format!("Erase failed: {e}"))?;
        Ok(format!("Erased {partition}"))
    }

    pub fn flash_scatter(
        device: &mut Device,
        scatter_path: &Path,
        selected: &[String],
    ) -> Result<String, String> {
        device.flash_scatter(
            scatter_path,
            |_, _| {},
            |name, _| selected.iter().any(|s| s == name),
            std::io::sink(),
            |_, _| {},
            |_, _| true,
        )
        .map_err(|e| format!("Flash failed: {e}"))?;
        Ok("Flash complete".into())
    }

    pub fn parse_scatter(scatter_path: &Path) -> Result<Vec<ScatterEntry>, String> {
        let content = std::fs::read_to_string(scatter_path)
            .map_err(|e| format!("Cannot read scatter file: {e}"))?;
        let mut entries = Vec::new();
        let mut current_name = String::new();
        let mut current_offset: u64 = 0;
        let mut current_size: u64 = 0;
        let mut current_file = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("partition_name") {
                if let Some(val) = trimmed.split_once('=').map(|v| v.1.trim().trim_matches('"')) {
                    current_name = val.to_string();
                }
            } else if trimmed.starts_with("linear_start_addr") || trimmed.starts_with("start_addr") {
                if let Some(val) = trimmed.split_once('=').map(|v| v.1.trim()) {
                    current_offset = u64::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                }
            } else if trimmed.starts_with("partition_size") || trimmed.starts_with("size") {
                if let Some(val) = trimmed.split_once('=').map(|v| v.1.trim()) {
                    current_size = u64::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                }
            } else if trimmed.starts_with("file_name") {
                if let Some(val) = trimmed.split_once('=').map(|v| v.1.trim().trim_matches('"')) {
                    current_file = val.to_string();
                }
            } else if trimmed.starts_with('}') {
                if !current_name.is_empty() {
                    entries.push(ScatterEntry {
                        name: current_name.clone(),
                        offset: current_offset,
                        size: current_size,
                        file_name: current_file.clone(),
                        enabled: true,
                    });
                    current_name.clear();
                    current_file.clear();
                    current_offset = 0;
                    current_size = 0;
                }
            }
        }
        Ok(entries)
    }
}

#[derive(Clone, Debug)]
pub struct ScatterEntry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub file_name: String,
    pub enabled: bool,
}
