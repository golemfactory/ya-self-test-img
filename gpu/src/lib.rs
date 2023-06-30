use std::process::Command;

use roxmltree::NodeId;
use serde_json::Value;

trait NodeLookup {
    fn lookup(&self, name: &str) -> anyhow::Result<NodeId>;
    fn lookup_child(&self, parent: NodeId, name: &str) -> anyhow::Result<NodeId>;
    fn node_text(&self, id: NodeId) -> anyhow::Result<String>;
}

impl<'a> NodeLookup for roxmltree::Document<'a> {
    fn lookup(&self, name: &str) -> anyhow::Result<NodeId> {
        Ok(self
            .descendants()
            .find(|d| d.tag_name().name() == name)
            .ok_or(anyhow::anyhow!("no node {name} in the XML"))?
            .id())
    }

    fn lookup_child(&self, parent: NodeId, name: &str) -> anyhow::Result<NodeId> {
        Ok(self
            .get_node(parent)
            .ok_or(anyhow::anyhow!("No node {parent:?}"))?
            .children()
            .find(|d| d.tag_name().name() == name)
            .ok_or(anyhow::anyhow!("no node {name} in the XML"))?
            .id())
    }

    fn node_text(&self, id: NodeId) -> anyhow::Result<String> {
        Ok(self
            .get_node(id)
            .ok_or(anyhow::anyhow!("No node {id:?}"))?
            .text()
            .ok_or(anyhow::anyhow!("Node {id:?} has no text"))?
            .to_string())
    }
}

fn clock_unit_to_hz(clock: &str) -> anyhow::Result<u64> {
    let parts: Vec<_> = clock.split_ascii_whitespace().collect();
    let freq = parts[0].parse::<f64>()?;
    Ok((freq
        * match parts[1].to_ascii_lowercase().as_str() {
            "ghz" => 1000000000.0,
            "mhz" => 1000000.0,
            "khz" => 1000.0,
            "hz" => 1.0,
            _ => anyhow::bail!("Invalid unit {}", parts[1]),
        })
    .round() as u64)
}

fn mem_unit_to_bytes(mem: &str) -> anyhow::Result<u64> {
    parse_size::parse_size(mem).map_err(|e| anyhow::anyhow!("parse_size: {e:?}"))
}

struct SmiOut {
    name: String,
    cuda_ver: String,
    clock_graphics_mhz: u64,
    clock_sm_mhz: u64,
    clock_mem_mhz: u64,
    clock_video_mhz: u64,
    mem_gib: f32,
}

fn nvidia_smi() -> anyhow::Result<SmiOut> {
    let xml_bytes = Command::new("nvidia-smi")
        .arg("-x")
        .arg("-q")
        .output()?
        .stdout;

    let xml_str = std::str::from_utf8(&xml_bytes)?;
    let doc = roxmltree::Document::parse(xml_str)?;

    let product_name_id = doc.lookup("product_name")?;
    let product_name = doc.node_text(product_name_id)?;

    let cuda_ver_id = doc.lookup("cuda_version")?;
    let cuda_ver = doc.node_text(cuda_ver_id)?;

    let max_clocks_id = doc.lookup("max_clocks")?;

    let max_graphics_clock_id = doc.lookup_child(max_clocks_id, "graphics_clock")?;
    let max_graphics_clock = doc.node_text(max_graphics_clock_id)?;

    let max_sm_clock_id = doc.lookup_child(max_clocks_id, "sm_clock")?;
    let max_sm_clock = doc.node_text(max_sm_clock_id)?;

    let max_mem_clock_id = doc.lookup_child(max_clocks_id, "mem_clock")?;
    let max_mem_clock = doc.node_text(max_mem_clock_id)?;

    let max_video_clock_id = doc.lookup_child(max_clocks_id, "video_clock")?;
    let max_video_clock = doc.node_text(max_video_clock_id)?;

    let fb_memory_id = doc.lookup("fb_memory_usage")?;
    let fb_total_memory_id = doc.lookup_child(fb_memory_id, "total")?;
    let fb_total_memory = doc.node_text(fb_total_memory_id)?;

    Ok(SmiOut {
        name: product_name,
        cuda_ver,
        clock_graphics_mhz: clock_unit_to_hz(&max_graphics_clock)? / 1000000,
        clock_sm_mhz: clock_unit_to_hz(&max_sm_clock)? / 1000000,
        clock_mem_mhz: clock_unit_to_hz(&max_mem_clock)? / 1000000,
        clock_video_mhz: clock_unit_to_hz(&max_video_clock)? / 1000000,
        mem_gib: mem_unit_to_bytes(&fb_total_memory)? as f32 / 1024.0 / 1024.0 / 1024.0,
    })
}

struct NvSettingsOut {
    mem_rate_max_gib_per_sec: u64,
    bus_width: u64,
    cuda_cores: u64,
}

fn nvidia_settings() -> anyhow::Result<NvSettingsOut> {
    let out_bytes = Command::new("nvidia-settings")
        .arg("--query")
        .arg("all")
        .output()?
        .stdout;
    let out_text = std::str::from_utf8(&out_bytes)?;

    let mut max_rate = 0;
    let mut bus_width = 0;
    let mut cores = 0;
    for line in out_text.lines() {
        let parts: Vec<_> = line.split_ascii_whitespace().collect();
        match parts.as_slice() {
            ["Attribute", "'CUDACores'", .., value] => {
                cores = value[0..value.len() - 1].parse::<u64>()?;
            }
            ["Attribute", "'GPUMemoryInterface'", .., value] => {
                bus_width = value[0..value.len() - 1].parse::<u64>()?;
            }
            ["Attribute", "'GPUPerfModes'", rem @ ..] => {
                for s in rem {
                    let parts: Vec<_> = s.split('=').collect();
                    if let ["memTransferRatemax", value] = parts.as_slice() {
                        let value = value.trim_end_matches(',').parse::<u64>()?;
                        if value > max_rate {
                            max_rate = value;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if max_rate == 0 {
        anyhow::bail!("memTransferRatemax attribute not found");
    }

    if bus_width == 0 {
        anyhow::bail!("GPUMemoryInterface attribute not found");
    }

    if cores == 0 {
        anyhow::bail!("CUDACores attribute not found");
    }

    Ok(NvSettingsOut {
        mem_rate_max_gib_per_sec: max_rate,
        bus_width,
        cuda_cores: cores,
    })
}

struct DebugOut {
    settings_out: String,
    smi_out: String,
    smi_xml: String,
}

fn debug_output() -> anyhow::Result<DebugOut> {
    let settings_bytes = Command::new("nvidia-settings")
        .arg("--query")
        .arg("all")
        .output()?
        .stdout;
    let settings_text = std::str::from_utf8(&settings_bytes)?;

    let smi_text_bytes = Command::new("nvidia-smi").arg("-q").output()?.stdout;
    let smi_text_text = std::str::from_utf8(&smi_text_bytes)?;

    let smi_xml_bytes = Command::new("nvidia-smi")
        .arg("-x")
        .arg("-q")
        .output()?
        .stdout;
    let smi_xml_text = std::str::from_utf8(&smi_xml_bytes)?;

    Ok(DebugOut {
        settings_out: settings_text.into(),
        smi_out: smi_text_text.into(),
        smi_xml: smi_xml_text.into(),
    })
}

fn gpu_status() -> anyhow::Result<Value> {
    let smi_out = nvidia_smi()?;
    let settings_out = nvidia_settings()?;

    Ok(serde_json::json!({
        "model": smi_out.name,
        "cuda": {
            "enabled": true,
            "cores": settings_out.cuda_cores,
            "version": smi_out.cuda_ver,
        },
        "clocks": {
            "graphics.mhz": smi_out.clock_graphics_mhz,
            "memory.mhz": smi_out.clock_mem_mhz,
            "sm.mhz": smi_out.clock_sm_mhz,
            "video.mhz": smi_out.clock_video_mhz,
        },
        "memory": {
            "bandwidth.gib": settings_out.mem_rate_max_gib_per_sec * settings_out.bus_width / (1000 * 8),
            "total.gib": smi_out.mem_gib,
        }
    }))
}

pub fn system_info() -> anyhow::Result<Value> {
    let gpu = gpu_status();

    Ok(if gpu.is_ok() {
        serde_json::json!({
            "gpu": gpu_status()?,
        })
    } else {
        match debug_output() {
            Ok(dbg) => serde_json::json!({
                "settings_out": dbg.settings_out,
                "smi_text": dbg.smi_out,
                "smi_xml": dbg.smi_xml,
            }),
            Err(e) => serde_json::json!({
                "err_debug": e.to_string(),
                "err_main": gpu.unwrap_err().to_string(),
            }),
        }
    })
}
