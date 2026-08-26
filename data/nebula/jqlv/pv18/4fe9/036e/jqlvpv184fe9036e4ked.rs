// Run one command, verdict from the exit status; stderr only matters on failure.
fn run(args: &[&str]) -> Result<String, String> {
  let out = Command::new(args[0]).args(&args[1..]).output();
  match out {
    Err(e) => Err(format!("could not run {}: {}", args[0], e)),
    Ok(o) => {
      let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
      if o.status.success() { Ok(text) } else { Err(text.trim().to_string()) }
    }
  }
}

if !Path::new("/run/systemd/system").exists() { return "ERROR: this host does not run systemd - use the supervised Run control instead.".to_string(); }
match run(&["sudo", "systemctl", "stop", &servicename]) {
  Ok(_) => "OK".to_string(),
  Err(e) => format!("ERROR: stop failed: {}", e),
}