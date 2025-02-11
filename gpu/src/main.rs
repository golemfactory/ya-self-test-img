use golem_gpu_info::GpuDetectionBuilder;
use serde_json::json;
use std::error::Error;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let arg = env::args_os().nth(1);
    let gpu = GpuDetectionBuilder::default()
        .force_cuda()
        .unstable_props()
        .init()?
        .detect()?;

    let sys_info = json!({"gpu": gpu});

    if let Some(path) = arg {
        fs::write(path, sys_info.to_string())?
    } else {
        print!("{}", sys_info);
    }
    Ok(())
}
