use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use penumbra_mtk::device::{Device, DeviceBuilder};
use penumbra_mtk::port::{PortBackend, PortType};
use penumbra_mtk::da::{BootMode, ScatterFile};
use penumbra_mtk::traits::{ProgressCallback, Reader, Writer};
use penumbra_mtk::activity::DeviceActivity;
use penumbra_mtk::{DeviceLog, DevInfo, Result as MtkResult};

use state::{LogLevel, OutputLog};

pub enum BackendCommand {
    Connect { backend: PortBackend },
    Disconnect,
    FlashScatter { scatter_path: PathBuf, files: Vec<(String, PathBuf)> },
    ReadPartition { name: String, output_path: PathBuf },
    WritePartition { name: String, input_path: PathBuf },
    ErasePartition { name: String },
    Reboot { mode: BootMode },
    Shutdown,
}

pub enum BackendEvent {
    Connected(DeviceInfo),
    Disconnected,
    Progress { current: usize, total: usize, message: String },
    Log(LogLevel, String),
    Error(String),
    Finished,
}

#[derive(Clone)]
pub struct DeviceInfo {
    pub chip_name: String,
    pub hw_code: u32,
    pub storage_type: String,
    pub sbc: bool,
    pub sla: bool,
    pub daa: bool,
}

pub struct MtkBackend {
    cancel: Arc<AtomicBool>,
}

impl MtkBackend {
    pub fn new() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn reset_cancel(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn connect(
        &self,
        backend: PortBackend,
        da_data: &[u8],
        log: &OutputLog,
    ) -> Result<DeviceInfo, String> {
        let port = PortType::find_and_open(None, None, backend)
            .map_err(|e| format!("Port discovery failed: {e}"))?
            .ok_or_else(|| "No MediaTek device found".to_string())?;

        let mut device = DeviceBuilder::new(port)
            .with_da_data(da_data.to_vec())
            .build()
            .map_err(|e| format!("Device build failed: {e}"))?;

        device.init().map_err(|e| format!("Device init failed: {e}"))?;

        let devinfo = device.devinfo();
        let chip = devinfo.chip();

        log.push(LogLevel::Info, format!("Connected: {}", chip.map(|c| c.name()).unwrap_or("Unknown")), &mut ());
        log.push(LogLevel::Info, format!("HW Code: 0x{:08X}", devinfo.hw_code()), &mut ());

        Ok(DeviceInfo {
            chip_name: chip.map(|c| c.name().to_string()).unwrap_or_default(),
            hw_code: devinfo.hw_code(),
            storage_type: format!("{:?}", device.get_storage().map(|s| s.storage_type()).unwrap_or(penumbra_mtk::storage::StorageType::Unknown)),
            sbc: devinfo.sbc_enabled(),
            sla: devinfo.sla_enabled(),
            daa: devinfo.daa_enabled(),
        })
    }
}