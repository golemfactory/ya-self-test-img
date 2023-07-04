use std::{env, fs};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let sys_info = ya_self_test_gpu::system_info()?;

    if args.len() == 2 {
        fs::write(&args[1], sys_info.to_string())?
    } else {
        print!("{}", sys_info);
    }
    Ok(())
}
