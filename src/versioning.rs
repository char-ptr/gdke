use std::{
    io::{Stdin, Stdout},
    path::Path,
    process::{Command, Stdio},
};
fn check_gd_ver(exe: Path) -> anyhow::Result<semver::Version> {
    assert!(exe.exists());
    let stdo = Command::new(exe)
        .arg("-V")
        .arg("-s")
        .arg("random-no-way-a-game-has-this-btw")
        .stdout(Stdio::null())
        .output()?;
    stdo.stdout
}
