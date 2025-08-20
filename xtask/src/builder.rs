use crate::{config::*, utils::run_prog};
use std::{fs::OpenOptions, io::Write};

static TARGET: &str = "aarch64-unknown-none-softfloat";

fn build_component(c: &Component, b: &BuildScript) -> Result<(), String> {
    info!("[INFO]     Builing {:?}...", c.name);

    run_prog(
        "cargo",
        &[
            "build",
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
    )
}

pub fn prepare_cpio(b: &Vec<Component>, to: &str) -> Result<(), String> {
    let mut files = Vec::with_capacity(b.len() - 1);
    let bin_out = format!(
        "{}{}{}{}",
        env!("CARGO_WORKSPACE_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    );

    info!("{}", env!("CARGO_MANIFEST_DIR"));

    for c in b {
        files.push(format!("{bin_out}/{}", c.name));
    }

    let mut out = Vec::new();

    run_prog(
        "cpio",
        &["-ocv"],
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
    for comp in &c.component {
        build_component(comp, c)?;
    }

    prepare_cpio(&c.component, "/tmp/archive.cpio")?;
    build_component(
        &Component {
            name: "roottask".to_string(),
        },
        c,
    )?;

    build_kernel(c)
}

pub fn run(c: BuildScript) -> Result<(), String> {
    build(&c)?;

    info!("[INFO]     Running example...");

    run_prog(
        "cargo",
        &[
            "run",
            "-p",
            "sam_kernel",
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
            "--features",
            &c.board,
        ],
        None,
        None,
        Some(&[("BOARD_TYPE", c.board.as_str())]),
    )
}
