use std::{process::{Command, exit}};
use bebop_tools as bebop;

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status().unwrap_or_else(|e| {
        eprintln!("failed to start {cmd}: {e}");
        exit(1)
    });

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}

fn bebop_generate() {
    bebop::download_bebopc(std::path::PathBuf::from("target"));

    bebop::build_schema_dir("messages/schemas", "messages", &bebop::BuildConfig::default());

}

fn main() {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some("docker:up") => run("docker", &["compose", "up", "-d"]),
        Some("docker:down") => run("docker", &["compose", "down"]),
        Some("docker:logs") => run("docker", &["compose", "logs", "-f"]),
        Some("bebop:generate") => bebop_generate(),
        _ => {
            eprintln!("tasks: docker:up | docker:down | docker:lobs");
            exit(1);
        }
    }
}
