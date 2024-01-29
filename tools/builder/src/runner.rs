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

pub fn build_kernel() -> Result<(), String> {
    info!("Building kernel...",);

    let mut child = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("sam_kernel")
        .arg("--target")
        .arg(TARGET)
        .arg("--features")
        .arg("qemu")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|x| format!("Failed to run cargo: {}", x))?;

    let e = child
        .wait()
        .map_err(|x| format!("Failed to build: {}", x))?;

    if !e.success() {
        Err(String::from(format!("Build failed with an error {e}")))
    } else {
        Ok(())
    }
}

fn do_build_component(name: &String) -> Result<(), String> {
    info!("Building component '{name}'...",);

    let mut child = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg(name)
        .arg("--target")
        .arg(TARGET)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|x| format!("Failed to run cargo: {}", x))?;

    let e = child
        .wait()
        .map_err(|x| format!("Failed to build: {}", x))?;

    if !e.success() {
        Err(String::from(format!("Build failed with an error {e}")))
    } else {
        Ok(())
    }
}

fn compile_idl(impls: &String, server: bool) -> Result<String, String> {
    info!(
        "Compiling '{}' part of interface {:?}...",
        if server { "server" } else { "transport" },
        impls
    );

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("ridl")
        .arg(if server { "server" } else { "transport" })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(format!("{IDLS_DIR}/{impls}.idl"))
        .spawn()
        .map_err(|x| format!("Failed to run ridl compiler {}", x))?;

    let exit = child
        .wait()
        .map_err(|_| format!("Failed to run ridl {}", impls))?;

    if !exit.success() {
        return Err(format!("Cargo failed with: {exit}"));
    }

    let out = std::io::read_to_string(child.stdout.unwrap())
        .map_err(|_| String::from("Failed to read ridl stdout"))?;

    assert!(out.len() != 0);
    Ok(out)
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

    let mut child = Command::new("cpio")
        .arg("-ocv")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|x| format!("Failed to run cargo: {}", x))?;

    child
        .stdin
        .take()
        .unwrap()
        .write(
            files
                .iter()
                .fold(String::new(), |mut s: String, x: &String| {
                    s.push_str(format!("{x}\n").as_str());
                    s
                })
                .as_str()
                .as_bytes(),
        )
        .map_err(|x| format!("Failed to write to cpio intput: {x}"))?;

    // NOTE: it's importnant to take stdout first, since otherwise process will
    // just block forever
    let mut buf = Vec::new();

    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut buf)
        .map_err(|x| format!("Failed to read cpio output {x}"))?;

    let exit = child
        .wait()
        .map_err(|x| format!("Failed to run cpio {}", x))?;

    if !exit.success() {
        return Err(format!("cpio failed with: {exit}"));
    }

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(to)
        .map_err(|_| String::from("Failed to create file cpio"))?;

    file.write(buf.as_slice())
        .map_err(|x| format!("Failed to write to file: {x}"))?;

    Ok(())
}

pub fn build_component(c: &Component) -> Result<(), String> {
    info!("Builing {:?}...", c.name);

    if let Some(ref impls) = c.implements {
        for i in impls {
            if i.as_str() != "init" {
                let out = compile_idl(i, false)?;

                let mut file = OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .open(format!("{INTERFACE_IMPL_DIR}/client/{}.rs", c.name))
                    .map_err(|_| String::from("Failed to create file for compiled idl"))?;

                file.write(out.as_str().as_bytes())
                    .map_err(|x| format!("Failed to write to file: {}", x))?;

                let out = compile_idl(i, true)?;

                let mut file = OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .open(format!("{INTERFACE_IMPL_DIR}/server/{}.rs", c.name))
                    .map_err(|_| String::from("Failed to create file for compiled idl"))?;

                file.write(out.as_str().as_bytes())
                    .map_err(|x| format!("Failed to write to file: {}", x))?;
            }
        }
    }

    do_build_component(&c.name)?;

    Ok(())
}
