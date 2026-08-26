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
let store = DataStore::new().root;
let nbdir = store.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let assets = store.join("nebula").join("_ASSETS");
let unit = std::fs::read_to_string(assets.join("service.txt")).unwrap();
let unit = unit.replace("SERVICENAME", &servicename);
let unit = unit.replace("ROOTDIR", &root.canonicalize().unwrap().into_os_string().into_string().unwrap());

let tmp = root.join("networks").join(&servicename).join(&(servicename.to_owned()+".service"));
std::fs::write(&tmp, &unit).unwrap();
let tmp = tmp.into_os_string().into_string().unwrap();
let dest = "/etc/systemd/system/".to_string()+&servicename+".service";

if let Err(e) = run(&["sudo", "mv", &tmp, &dest]) { return format!("ERROR: could not install the unit file: {}", e); }
if let Err(e) = run(&["sudo", "systemctl", "daemon-reload"]) { return format!("ERROR: daemon-reload failed: {}", e); }
if let Err(e) = run(&["sudo", "systemctl", "enable", &servicename]) { return format!("ERROR: enable failed: {}", e); }
"OK".to_string()