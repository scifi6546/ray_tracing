use lib_minya::{prelude::ParallelImage, ray_tracer::RayTracer};
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
    time::Instant,
};

fn get_commit_id() -> String {
    let command = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run command");
    String::from_utf8(command.stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn main() {
    let mut write_file = File::options()
        .append(true)
        .create(true)
        .open("./write_info.txt")
        .expect("failed to open buffer");
    let commit_id = get_commit_id();
    println!("commit id: \"{}\"", commit_id);
    let rt = RayTracer::builder().set_scenario("Fast Oct Tree Sinnoh".to_string());
    let start = Instant::now();
    let rt = rt.build();
    let construct_elapsed = start.elapsed();
    let mut parallel = ParallelImage::new_black(1024, 1024);
    let start = Instant::now();
    rt.trace_image(&mut parallel);
    let rendering_elapsed = start.elapsed();

    println!("elapsed time: {}ms", construct_elapsed.as_millis());
    println!("frame render time: {}", rendering_elapsed.as_millis());
    write!(
        write_file,
        "{}\t{}\t{}",
        commit_id,
        construct_elapsed.as_millis(),
        rendering_elapsed.as_millis()
    )
    .expect("failed to write log");
}
