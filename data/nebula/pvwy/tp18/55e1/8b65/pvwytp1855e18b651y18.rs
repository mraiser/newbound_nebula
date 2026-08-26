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
let unit = "/etc/systemd/system/".to_string()+&servicename+".service";
if !Path::new(&unit).exists() { return "OK".to_string(); }
// best-effort stop/disable, then the unit file must actually go away
let _x = run(&["sudo", "systemctl", "stop", &servicename]);
let _x = run(&["sudo", "systemctl", "disable", &servicename]);
if let Err(e) = run(&["sudo", "rm", &unit]) { return format!("ERROR: could not remove the unit file: {}", e); }
if let Err(e) = run(&["sudo", "systemctl", "daemon-reload"]) { return format!("ERROR: daemon-reload failed: {}", e); }
"OK".to_string()