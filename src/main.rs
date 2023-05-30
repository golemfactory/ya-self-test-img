fn main() -> anyhow::Result<()> {
    let sys_info = ya_self_test::system_info()?;

    print!("{}", sys_info.to_string());

    Ok(())
}
