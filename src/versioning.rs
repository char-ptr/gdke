use std::{
    io::{BufRead, BufReader, Cursor, Stdin, Stdout},
    path::Path,
    process::{Command, Stdio},
};
fn check_gd_ver(exe: &Path) -> anyhow::Result<String> {
    assert!(exe.exists());
    let stdo = Command::new(exe)
        .arg("-V")
        .arg("-s")
        .arg("random-no-way-a-game-has-this-btw")
        .stdout(Stdio::null())
        .output()?;
    let bufr = Cursor::new(stdo.stdout);

    Ok(bufr
        .lines()
        .next()
        .ok_or(anyhow::anyhow!("unable to read version"))??)
}
