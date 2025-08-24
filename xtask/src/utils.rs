use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

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

    let mut err = Vec::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut err)
        .map_err(|_| "Failed to read stderr of a process")?;
    std::io::stderr().write(err.as_slice()).unwrap();

    // Cargo prints a lot of stuff to stderr for some reason, but I like to perverse warnings
    // during build
    if !exit.success() {
        return Err(format!("{name} failed with: {exit}"));
    }

    Ok(())
}
