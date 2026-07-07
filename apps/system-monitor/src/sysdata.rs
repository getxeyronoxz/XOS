use sysinfo::{System, Disks, Networks};

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
}

#[derive(Clone)]
pub struct SystemStats {
    pub cpu_global: f32,
    pub cpu_cores: Vec<f32>,
    pub mem_total: u64,
    pub mem_used: u64,
    pub disks: Vec<(String, u64, u64)>, // (mount_point, total, used)
    pub net_rx: u64,
    pub net_tx: u64,
    pub processes: Vec<ProcessInfo>,
}

pub struct SysDataLoader {
    sys: System,
    last_net_rx: u64,
    last_net_tx: u64,
}

impl SysDataLoader {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let networks = Networks::new_with_refreshed_list();
        let mut initial_rx = 0;
        let mut initial_tx = 0;
        for (_, data) in &networks {
            initial_rx += data.received();
            initial_tx += data.transmitted();
        }

        Self {
            sys,
            last_net_rx: initial_rx,
            last_net_tx: initial_tx,
        }
    }

    pub fn fetch_stats(&mut self) -> SystemStats {
        self.sys.refresh_all();

        // CPU
        let cpu_global = self.sys.global_cpu_info().cpu_usage();
        let cpu_cores = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

        // Memory
        let mem_total = self.sys.total_memory();
        let mem_used = self.sys.used_memory();

        // Disks (using Disks struct in sysinfo 0.30)
        let mut disks = Vec::new();
        let disks_refreshed = Disks::new_with_refreshed_list();
        for disk in &disks_refreshed {
            let mount = disk.mount_point().to_string_lossy().into_owned();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            disks.push((mount, total, used));
        }

        // Networks (using Networks struct in sysinfo 0.30)
        let mut current_rx: u64 = 0;
        let mut current_tx: u64 = 0;
        let networks_refreshed = Networks::new_with_refreshed_list();
        for (_, data) in &networks_refreshed {
            current_rx += data.received();
            current_tx += data.transmitted();
        }

        let rx_diff = current_rx.saturating_sub(self.last_net_rx);
        let tx_diff = current_tx.saturating_sub(self.last_net_tx);
        self.last_net_rx = current_rx;
        self.last_net_tx = current_tx;

        // Processes
        let mut processes = Vec::new();
        for (pid, process) in self.sys.processes() {
            processes.push(ProcessInfo {
                pid: pid.to_string(),
                name: process.name().to_string(),
                cpu: process.cpu_usage(),
                memory: process.memory(),
            });
        }
        
        // Sort processes by CPU usage descending
        processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        // Keep top 50
        processes.truncate(50);

        SystemStats {
            cpu_global,
            cpu_cores,
            mem_total,
            mem_used,
            disks,
            net_rx: rx_diff,
            net_tx: tx_diff,
            processes,
        }
    }
}
