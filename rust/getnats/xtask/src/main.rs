use bebop_tools as bebop;
use std::process::{Command, exit};

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status().unwrap_or_else(|e| {
        eprintln!("failed to start {cmd}: {e}");
        exit(1)
    });

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}

fn build() {
    bebop_generate();
    run("cargo", &["build"]);
}

fn build_rel() {
    bebop_generate();
    run("cargo", &["build", "--release"]);
}

fn bebop_generate() {
    bebop::download_bebopc(std::path::PathBuf::from("target"));

    bebop::build_schema_dir(
        "messages/schemas",
        "messages",
        &bebop::BuildConfig::default(),
    );
}

type TaskFunc = fn();

const TASKS: &[(&str, TaskFunc)] = &[
    ("build", build),
    ("build:rel", build_rel),
    ("docker:up", || run("docker", &["compose", "up", "-d"])),
    ("docker:down", || run("docker", &["compose", "down"])),
    ("docker:logs", || run("docker", &["compose", "logs", "-f"])),
    ("bebop:generate", bebop_generate),
];

fn print_tasks() {
    let names: Vec<&str> = TASKS.iter().map(|(name, _)| *name).collect();
    eprintln!("tasks: {}", names.join(" | "));
}

fn main() {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some(task_name) => match TASKS.iter().find(|(name, _)| *name == task_name) {
            Some((_, task_fn)) => task_fn(),
            None => {
                eprintln!("unknown task: {task_name}");
                print_tasks();
                exit(1);
            }
        },
        None => {
            print_tasks();
            exit(1);
        }
    }
}
