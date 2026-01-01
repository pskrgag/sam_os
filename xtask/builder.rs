use crate::{config::*, utils::run_prog};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::{
    fs::{read_dir, symlink_metadata, OpenOptions},
    io::Write,
};

static TARGET: &str = "aarch64-unknown-none-softfloat";

fn has_manifest(path: &Path) -> std::io::Result<bool> {
    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && let Some(name) = path.file_name() && name == "Cargo.toml" {
            return Ok(true);
        }
    }

    Ok(false)
}

fn find_manifest_dir<P: AsRef<Path>>(
    dir: P,
    target: &str,
    results: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name() && name == target && has_manifest(&path)? {
                results.push(path.clone());
            }

            let metadata = symlink_metadata(&path)?;
            if !metadata.file_type().is_symlink() {
                find_manifest_dir(&path, target, results)?;
            }
        }
    }
    Ok(())
}

fn find_root_of<S: AsRef<str> + std::fmt::Display>(name: S) -> String {
    let mut res = vec![];

    find_manifest_dir(env!("CARGO_WORKSPACE_DIR"), name.as_ref(), &mut res).unwrap();

    if res.len() == 1 {
        res[0].as_os_str().to_str().unwrap().to_owned()
    } else if res.len() > 1 {
        panic!("There are multiply {name} packages {:?}", res)
    } else {
        panic!("There is no {name} package")
    }
}

fn find_manifest_of<S: AsRef<str> + std::fmt::Display>(name: S) -> String {
    format!("{}/Cargo.toml", find_root_of(name))
}

fn binary(name: &str) -> String {
    format!(
        "{}{}{}{}{name}",
        env!("CARGO_WORKSPACE_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    )
}

fn loader_binary() -> String {
    format!(
        "{}{}{}{}loader.bin",
        env!("CARGO_WORKSPACE_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    )
}

fn build_component(name: &str, b: &BuildScript, command: &str) -> Result<(), String> {
    info!("[INFO]     Building {:?}...", name);

    let opt_level = if let Some(ref lvl) = b.opt_level {
        format!("-C opt-level={}", lvl)
    } else {
        String::new()
    };

    run_prog(
        "cargo",
        &[
            command,
            "--manifest-path",
            &find_manifest_of(name),
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
        ],
        None,
        None,
        None,
        Some(&[
            ("BOARD_TYPE", b.board.as_str()),
            ("RUSTFLAGS", &format!("-C force-frame-pointers {opt_level} -C debug-assertions")),
            ("CARGO_TARGET_DIR", env!("CARGO_TARGET_DIR")),
        ]),
    )
}

// Returns path to kernel
fn build_test_kernel() -> Result<String, String> {
    let mut stdout = vec![];
    let regex = Regex::new(r".*Executable.*unittests.*\((.*)\)").unwrap();

    info!("[INFO]     Builing test kernel...");

    run_prog(
        "cargo",
        &[
            "test",
            "--no-run",
            "--manifest-path",
            &find_manifest_of("kernel"),
            "--target",
            TARGET,
            "--color=always",
        ],
        None,
        None,
        Some(&mut stdout),
        Some(&[("RUSTFLAGS", "-C force-frame-pointers")]),
    )?;

    let string = from_utf8(stdout.as_slice()).unwrap().to_owned();
    let kernel_name = regex
        .captures(&string)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_owned();

    Ok(format!("{}{kernel_name}", env!("CARGO_WORKSPACE_DIR"),))
}

fn build_loader(kernel: String) -> Result<(), String> {
    info!("[INFO]     Building loader...");

    // Do not raise opt level here, since for some reason rust generates weird ASM for linker var
    // access
    run_prog(
        "cargo",
        &[
            "build",
            "--manifest-path",
            &find_manifest_of("loader"),
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
            "-Z",
            "build-std=core",
        ],
        None,
        None,
        None,
        Some(&[
            ("RUSTFLAGS", "-C relocation-model=pie -C code-model=small -C debug-assertions"),
            ("KERNEL_PATH", &kernel),
            ("INIT_TASK_PATH", &binary("roottask")),
        ]),
    )?;

    run_prog(
        "llvm-objcopy",
        &[
            "-O",
            "binary",
            &binary("loader"),
            &format!("{}.bin", binary("loader")),
        ],
        None,
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

pub fn test() -> Result<(), String> {
    let kernel_name = build_test_kernel()?;
    build_loader(kernel_name)?;

    run_impl(false, None)
}

fn build_impl(c: &BuildScript, command: &str) -> Result<(), String> {
    for comp in &c.component {
        build_component(&comp.name, c, command)?;
    }

    prepare_cpio(&c.component, "/tmp/archive.cpio")?;
    build_component("roottask", c, command)?;

    build_component("kernel", c, command)?;
    build_loader(binary("kernel"))
}

fn run_impl(gdb: bool, c: Option<&BuildScript>) -> Result<(), String> {
    info!("[INFO]     Running example...");
    let bin = loader_binary();
    let mut args = vec![
        "-machine",
        "virt,gic-version=3",
        "-m",
        "1G",
        "-cpu",
        "cortex-a53",
        "-nographic",
        "-kernel",
        &bin,
    ];

    if let Some(c) = c
        && let Some(extra) = &c.extra_qemu_args
    {
        args.extend_from_slice(&extra.split_whitespace().collect::<Vec<_>>());
    }

    if gdb {
        args.extend_from_slice(&["-s", "-S"]);
    }

    info!("qemu-system-aarch64 {}", args.join(" "));
    run_prog(
        "qemu-system-aarch64",
        args.as_slice(),
        None,
        None,
        None,
        None,
    )
}

pub fn run(c: BuildScript, gdb: bool) -> Result<(), String> {
    build(&c)?;
    run_impl(gdb, Some(&c))
}

pub fn clippy(c: BuildScript) -> Result<(), String> {
    build_impl(&c, "clippy")
}
