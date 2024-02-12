use std::{
    io::{BufRead, Cursor, Read},
    path::Path,
    process::{Command, Stdio},
};
pub fn check_gd_ver(exe: &Path) -> anyhow::Result<String> {
    assert!(exe.exists());
    let stdo = Command::new(exe)
        .arg("--version")
        // .stderr(Stdio::null())
        .output()?;
    let mut bufr = Cursor::new(stdo.stdout);

    let mut out = String::new();
    bufr.read_to_string(&mut out)
        .map_err(|_| anyhow::anyhow!("unable to read version"))?;
    Ok(out.trim().to_string())
}
