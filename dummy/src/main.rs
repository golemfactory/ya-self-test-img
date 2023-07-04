use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    let sys_info = ya_self_test::system_info();

    if args.len() == 2 {
        fs::write(&args[1], sys_info.to_string())
            .expect("Unable to write file");
    } else {
        print!("{}", sys_info);
    }
}
