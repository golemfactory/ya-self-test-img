use serde_json::Value;
use sysinfo::{System, SystemExt};

pub fn system_info() -> anyhow::Result<Value> {
    let sys = System::new_all();
    let cpu_num = sys.cpus().len();
    let memory = sys.total_memory();

    Ok(serde_json::json!({
        "cpu.num": cpu_num,
        "mem.total": memory
    }))
}
