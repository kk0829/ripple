use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

const SERVICE_TYPE: &str = "_ripple._tcp.local.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub name: String,
    pub os_type: String,
    pub ip: String,
    pub port: u16,
}

pub type SharedDevices = Arc<RwLock<HashMap<String, DeviceInfo>>>;

pub struct SharedState {
    pub devices: SharedDevices,
    pub port: u16,
    pub my_fingerprint: String,
    pub my_name: String,
}

pub fn register(app: &tauri::AppHandle) {
    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown".into())
        .to_string_lossy()
        .to_string();

    let port = get_available_port();
    let my_fp = fingerprint();

    app.manage(SharedState {
        devices: Arc::new(RwLock::new(HashMap::new())),
        port,
        my_fingerprint: my_fp.clone(),
        my_name: hostname.clone(),
    });

    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let my_ip = get_local_ip();

    let properties: &[(&str, &str)] = &[
        (
            "device_name",
            Box::leak(hostname.clone().into_boxed_str()) as &str,
        ),
        ("os", std::env::consts::OS),
        (
            "fingerprint",
            Box::leak(my_fp.clone().into_boxed_str()) as &str,
        ),
    ];

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &format!("{}-{}", hostname, port),
        &format!("{}.local.", hostname),
        &my_ip as &str,
        port,
        properties,
    )
    .expect("Failed to create ServiceInfo");

    mdns.register(service_info)
        .expect("Failed to register mDNS service");
    tracing::info!("Registered mDNS service: {} on port {}", SERVICE_TYPE, port);

    let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse mDNS");
    let devices: SharedDevices = app.state::<SharedState>().devices.clone();
    let my_fingerprint = my_fp.clone();

    let app_handle = app.clone();
    std::thread::spawn(move || loop {
        match receiver.recv_timeout(std::time::Duration::from_secs(1)) {
            Ok(event) => match event {
                ServiceEvent::ServiceResolved(info) => {
                    let fp = info
                        .get_property_val_str("fingerprint")
                        .unwrap_or("")
                        .to_string();

                    if fp == my_fingerprint || fp.is_empty() {
                        continue;
                    }

                    let ip = info
                        .get_addresses()
                        .iter()
                        .next()
                        .map(|a| a.to_string())
                        .unwrap_or_default();

                    let device = DeviceInfo {
                        fingerprint: fp.clone(),
                        name: info
                            .get_property_val_str("device_name")
                            .unwrap_or(info.get_fullname())
                            .to_string(),
                        os_type: info
                            .get_property_val_str("os")
                            .unwrap_or("unknown")
                            .to_string(),
                        ip,
                        port: info.get_port(),
                    };

                    tracing::info!("Discovered device: {} ({})", device.name, device.ip);

                    {
                        let mut d = devices.blocking_write();
                        d.insert(fp.clone(), device.clone());
                    }

                    let _ = app_handle.emit("device-discovered", &device);
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    let mut d = devices.blocking_write();
                    let key = fullname.split('.').next().unwrap_or("");
                    d.retain(|_, v| {
                        let keep = v.name != key;
                        if !keep {
                            tracing::info!("Device removed: {}", v.name);
                            let _ = app_handle.emit("device-removed", &v.fingerprint);
                        }
                        keep
                    });
                }
                _ => {}
            },
            Err(e) => {
                if e.to_string().contains("timed out") {
                    continue;
                }
                tracing::error!("mDNS browse error: {}", e);
                break;
            }
        }
    });
}

fn get_available_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind(("0.0.0.0", 0))
        .map(|l| l.local_addr().unwrap().port())
        .unwrap_or(9700)
}

fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn fingerprint() -> String {
    use base64::Engine;
    let raw = format!(
        "{}-{}-{}",
        hostname::get().unwrap_or_default().to_string_lossy(),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes())
}
