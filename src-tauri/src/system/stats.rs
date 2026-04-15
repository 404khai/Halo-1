use std::sync::Mutex;
use std::thread;

use serde::Serialize;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, Networks, System};
use tauri::State;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatsPayload {
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub battery: BatteryStats,
    pub network: NetworkStats,
    pub volume: VolumeStats,
    pub notifications: NotificationStats,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    pub usage_percent: f32,
    pub available: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: f32,
    pub available: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatteryStats {
    pub percentage: Option<u8>,
    pub status: String,
    pub available: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStats {
    pub download_bytes_per_second: u64,
    pub upload_bytes_per_second: u64,
    pub connected: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VolumeStats {
    pub level_percent: Option<u8>,
    pub muted: bool,
    pub available: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStats {
    pub count: u32,
}

pub struct SystemStatsState {
    system: Mutex<System>,
    networks: Mutex<Networks>,
}

impl SystemStatsState {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        system.refresh_cpu_usage();
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        Self {
            system: Mutex::new(system),
            networks: Mutex::new(networks),
        }
    }
}

#[tauri::command]
pub fn get_system_stats(state: State<'_, SystemStatsState>) -> Result<SystemStatsPayload, String> {
    let cpu = read_cpu(&state)?;
    let memory = read_memory(&state)?;
    let network = read_network(&state)?;
    let battery = read_battery().unwrap_or_else(|_| BatteryStats {
        percentage: None,
        status: "Unavailable".to_string(),
        available: false,
    });
    let volume = read_volume().unwrap_or_else(|_| VolumeStats {
        level_percent: None,
        muted: false,
        available: false,
    });

    Ok(SystemStatsPayload {
        cpu,
        memory,
        battery,
        network,
        volume,
        notifications: NotificationStats { count: 0 },
    })
}

#[tauri::command]
pub fn set_volume_muted(muted: bool) -> Result<(), String> {
    write_volume_mute(muted)
}

fn read_cpu(state: &State<'_, SystemStatsState>) -> Result<CpuStats, String> {
    let mut system = state
        .system
        .lock()
        .map_err(|_| "Failed to acquire system state".to_string())?;

    system.refresh_cpu_usage();

    Ok(CpuStats {
        usage_percent: system.global_cpu_usage(),
        available: true,
    })
}

fn read_memory(state: &State<'_, SystemStatsState>) -> Result<MemoryStats, String> {
    let mut system = state
        .system
        .lock()
        .map_err(|_| "Failed to acquire memory state".to_string())?;

    system.refresh_memory();

    let used_bytes = system.used_memory();
    let total_bytes = system.total_memory();
    let usage_percent = if total_bytes == 0 {
        0.0
    } else {
        (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
    };

    Ok(MemoryStats {
        used_bytes,
        total_bytes,
        usage_percent,
        available: total_bytes > 0,
    })
}

fn read_network(state: &State<'_, SystemStatsState>) -> Result<NetworkStats, String> {
    let mut networks = state
        .networks
        .lock()
        .map_err(|_| "Failed to acquire network state".to_string())?;

    networks.refresh(true);

    let mut download_bytes_per_second = 0_u64;
    let mut upload_bytes_per_second = 0_u64;

    for (_, network) in &*networks {
        download_bytes_per_second += network.received() / 2;
        upload_bytes_per_second += network.transmitted() / 2;
    }

    Ok(NetworkStats {
        download_bytes_per_second,
        upload_bytes_per_second,
        connected: download_bytes_per_second > 0 || upload_bytes_per_second > 0,
    })
}

#[cfg(target_os = "windows")]
fn read_battery() -> Result<BatteryStats, String> {
    use windows::Devices::Power::Battery;

    let battery = Battery::AggregateBattery().map_err(|error| error.to_string())?;
    let report = battery.GetReport().map_err(|error| error.to_string())?;

    let full_charge = report
        .FullChargeCapacityInMilliwattHours()
        .ok()
        .and_then(|value| value.Value().ok());
    let remaining_charge = report
        .RemainingCapacityInMilliwattHours()
        .ok()
        .and_then(|value| value.Value().ok());
    let charge_rate = report
        .ChargeRateInMilliwatts()
        .ok()
        .and_then(|value| value.Value().ok());

    let percentage = match (remaining_charge, full_charge) {
        (Some(remaining), Some(full)) if full > 0 => {
            Some(((remaining as f64 / full as f64) * 100.0).round().clamp(0.0, 100.0) as u8)
        }
        _ => None,
    };

    let status = match charge_rate {
        Some(rate) if rate > 0 => "Charging",
        Some(rate) if rate < 0 => "Discharging",
        Some(_) => "Idle",
        None => "Unavailable",
    }
    .to_string();

    Ok(BatteryStats {
        percentage,
        available: percentage.is_some(),
        status,
    })
}

#[cfg(not(target_os = "windows"))]
fn read_battery() -> Result<BatteryStats, String> {
    Ok(BatteryStats {
        percentage: None,
        status: "Unavailable".to_string(),
        available: false,
    })
}

#[cfg(target_os = "windows")]
fn read_volume() -> Result<VolumeStats, String> {
    // Graceful fallback until the Windows endpoint-volume bindings are wired correctly.
    Ok(VolumeStats {
        level_percent: None,
        muted: false,
        available: false,
    })
}

#[cfg(target_os = "windows")]
fn write_volume_mute(_muted: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn read_volume() -> Result<VolumeStats, String> {
    Ok(VolumeStats {
        level_percent: None,
        muted: false,
        available: false,
    })
}

#[cfg(not(target_os = "windows"))]
fn write_volume_mute(_muted: bool) -> Result<(), String> {
    Ok(())
}
