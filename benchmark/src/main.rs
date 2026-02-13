use lib_minya::{prelude::ParallelImage, ray_tracer::RayTracer};
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
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
fn trace_image(tracer: &RayTracer, image: &mut ParallelImage) -> Duration {
    let start = Instant::now();
    tracer.trace_image(image);
    start.elapsed()
}
fn calculate_mean(elapsed: &[Duration]) -> Duration {
    let mut total = Duration::ZERO;
    for e in elapsed {
        total = total.checked_add(*e).unwrap();
    }
    total / elapsed.len() as u32
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
    println!("elapsed time: {}ms", construct_elapsed.as_millis());
    let mut parallel = ParallelImage::new_black(1024, 1024);
    let num_runs = 100;
    let mut elapsed_array = Vec::with_capacity(num_runs);
    for _ in 0..num_runs {
        let rendering_elapsed = trace_image(&rt, &mut parallel);
        elapsed_array.push(rendering_elapsed);
    }
    let mean_rendering = calculate_mean(&elapsed_array);
    println!("frame render time: {}", mean_rendering.as_millis());
    write!(
        write_file,
        "\n{}\t{}\t{}",
        commit_id,
        construct_elapsed.as_millis(),
        mean_rendering.as_millis()
    )
    .expect("failed to write log");
}
