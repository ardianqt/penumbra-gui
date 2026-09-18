use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use penumbra_mtk::device::DeviceBuilder;
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

    pub fn connect(da_bytes: &[u8]) -> Result<(DeviceInfo, String), String> {
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

        Ok((info, format!("Connected: {chip_name}")))
    }
}