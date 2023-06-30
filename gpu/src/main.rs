fn main() -> anyhow::Result<()> {
    let sys_info = ya_self_test_gpu::system_info()?;

    print!("{}", sys_info);

    Ok(())
}
