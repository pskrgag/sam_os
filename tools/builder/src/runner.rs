use crate::toml::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

static IDLS_DIR: &str = concat!(
    env!("CARGO_RUSTC_CURRENT_DIR"),
    "/src/userspace/interfaces/idls/"
);

static INTERFACE_IMPL_DIR: &str = concat!(
    env!("CARGO_RUSTC_CURRENT_DIR"),
    "/src/userspace/interfaces/src/"
);

static TARGET: &str = "aarch64-unknown-none-softfloat";

pub fn run_prog(
    name: &str,
    args: &[&str],
    stdin: Option<&[u8]>,
    stdout: Option<&mut Vec<u8>>,
    env: Option<&[(&str, &str)]>,
) -> Result<(), String> {
    let mut child = Command::new(name);
    let mut child = child.args(args).stderr(Stdio::piped());

    if stdin.is_some() {
        child = child.stdin(Stdio::piped());
    }

    if stdout.is_some() {
        child = child.stdout(Stdio::piped());
    }

    if let Some(env) = env {
        for i in env {
            child = child.env(i.0, i.1);
        }
    }

    let mut child = child
        .spawn()
        .map_err(|x| format!("Failed to run {name}: {x}"))?;

    if let Some(stdin) = stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write(stdin)
            .map_err(|x| format!("Failed to write to {name} intput: {x}"))?;
    }

    if let Some(stdout) = stdout {
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(stdout)
            .map_err(|x| format!("Failed to read cpio output {x}"))?;
    }

    let exit = child
        .wait()
        .map_err(|x| format!("Failed to run {name} {x}"))?;

    // Cargo prints a lot of stuff to stderr for some reason, but I like to preverse warnings
    // during build
    // if !exit.success() {
    let mut err = Vec::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut err)
        .map_err(|_| "Failed to read stderr of a process")?;

    std::io::stderr().write(err.as_slice()).unwrap();

    // return Err(format!("{name} failed with: {exit}"));
    // }

    if !exit.success() {
        Err(String::new())
    } else {
        Ok(())
    }
}

pub fn build_kernel(b: &BuildScript, init_name: &String) -> Result<(), String> {
    info!("[CARGO]    Building kernel...",);

    let ldpath = "src/kernel/src/arch/aarch64/aarch64-qemu.ld";
    let flag = format!("-DCONFIG_BOARD_{}", b.board.to_uppercase());

    run_prog(
        "cpp",
        &[ldpath, flag.as_str(), "-o", "/tmp/tmp.ld"],
        None,
        None,
        None,
    )
    .unwrap();

    let mut out = Vec::new();
    run_prog(
        "cargo",
        &[
            "build",
            "-p",
            "sam_kernel",
            "--target",
            TARGET,
            "--features",
            &b.board.to_string(),
            "--color=always",
        ],
        None,
        Some(&mut out),
        Some(&[
            (
                "RUSTFLAGS",
                "-C link-arg=--script=/tmp/tmp.ld -C opt-level=2 -C force-frame-pointers",
            ),
            ("BOARD_TYPE", b.board.as_str()),
            ("SAMOS_INIT_NAME", init_name),
        ]),
    )
}

fn do_build_component(name: &String, b: &BuildScript) -> Result<(), String> {
    info!("[CARGO]    Compiling component \"{name}\"...",);

    run_prog(
        "cargo",
        &[
            "build",
            "-p",
            name.as_str(),
            "--target",
            TARGET,
            "--color=always",
            "--quiet",
        ],
        None,
        None,
        Some(&[
            ("BOARD_TYPE", b.board.as_str()),
            ("RUSTFLAGS", "-C opt-level=2"),
        ]),
    )
}

pub fn compile_idl(i: &String, server: bool) -> Result<String, String> {
    info!(
        "[RIDL]     Compiling '{}' part of interface {:?}...",
        if server { "server" } else { "transport" },
        i
    );

    let mut out = Vec::new();
    run_prog(
        "cargo",
        &[
            "run",
            "--color=always",
            "-p",
            "ridl",
            if server { "server" } else { "transport" },
            format!("{IDLS_DIR}/{i}").as_str(),
        ],
        None,
        Some(&mut out),
        None,
    )?;

    let s = std::str::from_utf8(out.as_slice()).unwrap();
    Ok(s.to_string())
}

pub fn prepare_cpio(b: &Vec<Component>, to: &str) -> Result<(), String> {
    let mut files = Vec::with_capacity(b.len() - 1);
    let bin_out = format!(
        "{}{}{}{}",
        env!("CARGO_RUSTC_CURRENT_DIR"),
        "/target/",
        TARGET,
        "/debug/",
    );

    for c in b {
        if let Some(ref impls) = c.implements {
            if impls.iter().find(|x| x.as_str() == "init").is_some() {
                continue;
            }
        }

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
                .as_str()
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

pub fn prepare_idls() -> Result<(), String> {
    let paths = std::fs::read_dir(IDLS_DIR).unwrap();

    let mut server_mod = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(format!("{INTERFACE_IMPL_DIR}/server/mod.rs"))
        .map_err(|_| String::from("Failed to create file for mod"))?;

    let mut client_mod = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(format!("{INTERFACE_IMPL_DIR}/client/mod.rs"))
        .map_err(|_| String::from("Failed to create file for mod"))?;

    for i in paths {
        let i = i.unwrap();
        let path = i.path();
        let i = path.file_name().unwrap().to_str().unwrap().to_string();
        let file_strep = path.file_stem().unwrap().to_str().unwrap();

        let out = compile_idl(&i, true)?;

        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{INTERFACE_IMPL_DIR}/server/{}.rs", file_strep))
            .map_err(|_| String::from("Failed to create file for compiled idl"))?;

        file.write(out.as_str().as_bytes())
            .map_err(|x| format!("Failed to write to file: {}", x))?;

        writeln!(server_mod, "pub mod {};", file_strep).unwrap();

        let out = compile_idl(&i, false)?;

        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{INTERFACE_IMPL_DIR}/client/{}.rs", file_strep))
            .map_err(|_| String::from("Failed to create file for compiled idl"))?;

        file.write(out.as_str().as_bytes())
            .map_err(|x| format!("Failed to write to file: {}", x))?;

        writeln!(client_mod, "pub mod {};", file_strep).unwrap();
    }

    Ok(())
}

pub fn build_component(c: &Component, b: &BuildScript) -> Result<(), String> {
    info!("[INFO]     Builing {:?}...", c.name);

    do_build_component(&c.name, b)
}
