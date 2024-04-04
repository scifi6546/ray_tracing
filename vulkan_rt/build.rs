use std::path::PathBuf;
use std::{
    fs::{create_dir, read_dir},
    path::Path,
    process::{exit, Command},
};

/// Source path of shaders
const SHADER_SRC_PATH: &str = "shaders/src";

/// Where Shaders are written to
const SHADER_BIN_PATH: &str = "shaders/bin";
fn get_output_path(output_shader_name: &str) -> PathBuf {
    Path::new(SHADER_BIN_PATH).join(output_shader_name)
}
fn create_output_directory(output_shader_name: &str) {
    let parent_path = Path::new(SHADER_BIN_PATH);
    if !parent_path.exists() {
        create_dir(parent_path).expect("failed to create parent path")
    }
    let output_path = get_output_path(output_shader_name);
    if !output_path.exists() {
        create_dir(output_path).unwrap();
    }
}
fn compile_directory(path: &Path) {
    if !path.is_dir() {
        eprintln!("path: {} is not directory", path.to_str().unwrap());
        exit(1)
    }
    let shader_name = path.file_name().expect("not regular file");
    create_output_directory(shader_name.to_str().unwrap());
    for file_res in read_dir(path).unwrap() {
        let file = file_res.unwrap();

        let shader_path = file.path().canonicalize().unwrap();

        let shader_path_str = shader_path.to_str().unwrap();
        let output_shader_name = file.file_name().to_str().unwrap().to_string() + ".spv";
        let output_shader_path =
            get_output_path(shader_name.to_str().unwrap()).join(&output_shader_name);
        let command_result = Command::new("glslang")
            .args([
                "-V",
                "-H",
                "--target-env",
                "vulkan1.3",
                shader_path_str,
                "-o",
                output_shader_path.to_str().unwrap(),
            ])
            .output();
        if command_result.is_err() {
            let err = command_result.err().unwrap();
            eprintln!("failed to compile shader: {}", err);
            exit(1)
        } else {
        }
        println!("cargo:rerun-if-changed={}", shader_path_str);
    }
}
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
        if file.file_type().expect("failed to get file type").is_dir() {
            compile_directory(file.path().as_path())
        }
    }
}
