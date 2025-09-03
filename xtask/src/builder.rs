use crate::{config::*, utils::run_prog};
use std::{fs::OpenOptions, io::Write};

static TARGET: &str = "aarch64-unknown-none-softfloat";

fn binary(name: &str) -> String {
    format!(
        "{}{}{}{}{name}",
        env!("CARGO_WORKSPACE_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    )
}

fn kernel_binary(name: &str) -> String {
    format!(
        "{}{}{}{}{name}.bin",
        env!("CARGO_WORKSPACE_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    )
}

fn build_component(c: &Component, b: &BuildScript, command: &str) -> Result<(), String> {
    info!("[INFO]     Builing {:?}...", c.name);

    run_prog(
        "cargo",
        &[
            command,
            "-p",
            c.name.as_str(),
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
        ],
        None,
        None,
        Some(&[("BOARD_TYPE", b.board.as_str())]),
    )
}

fn build_kernel(b: &BuildScript) -> Result<(), String> {
    info!("[INFO]     Builing kernel...");

    run_prog(
        "cargo",
        &[
            "build",
            "-p",
            "sam_kernel",
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
            "--features",
            &b.board,
        ],
        None,
        None,
        Some(&[("BOARD_TYPE", b.board.as_str())]),
    )?;

    run_prog(
        "llvm-objcopy",
        &[
            "-O",
            "binary",
            &binary("sam_kernel"),
            &format!("{}.bin", binary("sam_kernel")),
        ],
        None,
        None,
        None,
    )
}

pub fn prepare_cpio(b: &Vec<Component>, to: &str) -> Result<(), String> {
    let mut files = Vec::with_capacity(b.len() - 1);

    for c in b {
        files.push(binary(c.name.as_str()));
    }

    let mut out = Vec::new();

    run_prog(
        "cpio",
        &["-oc"],
        Some(
            files
                .iter()
                .fold(String::new(), |mut s: String, x: &String| {
                    s.push_str(format!("{x}\n").as_str());
                    s
                })
                .as_bytes(),
        ),
        Some(&mut out),
        None,
    )?;

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(to)
        .map_err(|_| String::from("Failed to create file cpio"))?;

    file.write(out.as_slice())
        .map_err(|x| format!("Failed to write to file: {x}"))?;

    Ok(())
}

pub fn build(c: &BuildScript) -> Result<(), String> {
    build_impl(c, "build")
}

fn build_impl(c: &BuildScript, command: &str) -> Result<(), String> {
    for comp in &c.component {
        build_component(comp, c, command)?;
    }

    prepare_cpio(&c.component, "/tmp/archive.cpio")?;
    build_component(
        &Component {
            name: "roottask".to_string(),
        },
        c,
        command,
    )?;

    build_kernel(c)
}

pub fn run(c: BuildScript, gdb: bool) -> Result<(), String> {
    build(&c)?;

    info!("[INFO]     Running example...");
    let bin = kernel_binary("sam_kernel");
    let mut args = vec![
        "-machine",
        "virt,gic-version=2",
        "-m",
        "1G",
        "-cpu",
        "cortex-a53",
        // "-smp",
        // "2",
        "-nographic",
        "-kernel",
        &bin,
        "-d",
        "int",
        "-D",
        "log1",
    ];

    if gdb {
        args.extend_from_slice(&["-s", "-S"]);
    }

    // qemu-system-aarch64 -machine virt,gic-version=2 -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel
    run_prog(
        "qemu-system-aarch64",
        args.as_slice(),
        None,
        None,
        Some(&[("BOARD_TYPE", c.board.as_str())]),
    )
}

pub fn clippy(c: BuildScript) -> Result<(), String> {
    build_impl(&c, "clippy")
}
