use std::{
    fs::{create_dir, read_dir},
    path::Path,
    process::{Command, ExitStatus},
};
const SHADER_SRC_PATH: &str = "shaders/src";
const SHADER_BIN_PATH: &str = "shaders/bin";
fn main() {
    println!("cargo:rerun-if-changed={}", SHADER_SRC_PATH);
    let shader_path = Path::new(SHADER_SRC_PATH);
    {
        let bin_path = Path::new(SHADER_BIN_PATH);
        if !bin_path.exists() {
            create_dir(bin_path).expect("failed to create bin path")
        }
    }

    for file_res in read_dir(shader_path).expect("shader path does not exist") {
        let file = file_res.expect("file does not exist");
        let file_path = file.path();
        let file_name = file_path.file_name().unwrap();
        let file_split = file_name.to_str().unwrap().split(".").collect::<Vec<_>>();
        let shader_type = file_split[1];
        let output_path = format!("{}/{}", SHADER_BIN_PATH, file_name.to_str().unwrap());
        let input_path = file_path.to_str().unwrap();
        let args = [
            &format!("-fshader-stage={}", shader_type),
            input_path,
            // "-fshader-stage=",
            // &shader_type,
            "-o",
            &output_path,
        ];

        let o = Command::new("glslc")
            .args(args)
            .output()
            .expect("failed to run glslc");
        if !o.status.success() {
            println!("args: {:#?}", args);
            let s = std::str::from_utf8(&o.stdout).unwrap();
            println!("{}", s);
            let s = std::str::from_utf8(&o.stderr).unwrap();
            println!("{}", s);
            std::process::exit(1);
        }
        println!("cargo:rerun-if-changed={}", input_path);
    }
}
