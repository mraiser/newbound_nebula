use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use flowlang::flowlang::file::read_properties::read_properties;
use std::fs;
use ndata::dataarray::DataArray;
use std::path::Path;
use flowlang::flowlang::system::system_call::system_call;

// ---- shared helpers (also used by start; the UI re-checks via info after an engine install) ----

/// Quote a path for a copy-paste shell command when it needs it.
pub fn nb_shell_quote(s: &str) -> String {
  if s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+:=@".contains(c)) { s.to_string() }
  else if cfg!(windows) { format!("\"{}\"", s) }
  else { format!("'{}'", s.replace('\'', "'\\''")) }
}

/// Is this process root (Unix) / elevated (Windows)? None when it can't tell.
pub fn nb_is_root() -> Option<bool> {
  if cfg!(windows) {
    // `net session` only succeeds in an elevated process
    return std::process::Command::new("net").arg("session")
      .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
      .status().ok().map(|s| s.success());
  }
  let out = std::process::Command::new("id").arg("-u").output().ok()?;
  Some(String::from_utf8_lossy(&out.stdout).trim() == "0")
}

/// Linux: does the engine binary carry cap_net_admin? None when getcap is missing.
pub fn nb_has_cap_net_admin(bin: &str) -> Option<bool> {
  for g in ["getcap", "/usr/sbin/getcap", "/sbin/getcap"] {
    if let Ok(out) = std::process::Command::new(g).arg(bin).output() {
      return Some(String::from_utf8_lossy(&out.stdout).contains("cap_net_admin"));
    }
  }
  None
}

/// What this OS needs before nebula can create its tunnel device, as plain
/// words plus (where one exists) the exact command that grants it.
/// {os, arch, ready (absent = can't tell), how, fix?, note?}
pub fn nb_tun_access(bin: &Path, config: &str) -> DataObject {
  let mut t = DataObject::new();
  let os = std::env::consts::OS;
  t.put_string("os", os);
  t.put_string("arch", std::env::consts::ARCH);
  let binstr = bin.canonicalize().map(|p| p.display().to_string()).unwrap_or(bin.display().to_string());
  let cfg = if config.is_empty() { "<network folder>/config.yml".to_string() } else { nb_shell_quote(config) };
  let root = nb_is_root();
  let ready: Option<bool>;
  match os {
    "linux" => {
      let cap = if bin.exists() { nb_has_cap_net_admin(&binstr) } else { None };
      ready = if root == Some(true) || cap == Some(true) { Some(true) } else { cap };
      t.put_string("how", "Linux lets a program create a network interface only when it has the cap_net_admin permission. Give it to the nebula engine once (it asks for your password), then press Run now again.");
      t.put_string("fix", &format!("sudo setcap cap_net_admin=+ep {}", nb_shell_quote(&binstr)));
      t.put_string("note", "Installing a new engine replaces this file, so run it again after every engine update. No setcap command? It comes with libcap: sudo apt install libcap2-bin (Debian, Ubuntu, Raspberry Pi OS); on NixOS add libcap to your packages.");
    }
    "macos" => {
      ready = root;
      t.put_string("how", "macOS lets only root create the tunnel's utun interface, and Newbound isn't running as root. Start the tunnel yourself in Terminal instead (it asks for your password; keep the window open):");
      t.put_string("fix", &format!("sudo {} -config {}", nb_shell_quote(&binstr), cfg));
      t.put_string("note", "Newbound can't install a launchd service for this yet, so the tunnel stops when you close Terminal or restart the Mac.");
    }
    "windows" => {
      ready = root;
      t.put_string("how", "Windows lets only an administrator create the tunnel's network adapter. Quit Newbound and start it again with \"Run as administrator\", then press Run now.");
      t.put_string("note", "nebula also needs the Wintun driver: keep wintun.dll from the Windows release next to nebula.exe.");
    }
    _ => {
      ready = root;
      t.put_string("how", "This system lets only root create the tunnel's network interface. Run Newbound as root, or start the tunnel yourself:");
      t.put_string("fix", &format!("sudo {} -config {}", nb_shell_quote(&binstr), cfg));
    }
  }
  if let Some(r) = ready { t.put_boolean("ready", r); }
  t
}

/// A certificate's notAfter (RFC 3339) via `nebula-cert print -json`; "" if unknown.
pub fn nb_cert_not_after(certbin: &Path, crt: &Path) -> String {
  let out = std::process::Command::new(certbin).args(["print", "-json", "-path", &crt.display().to_string()]).output();
  if let Ok(o) = out {
    let s = String::from_utf8_lossy(&o.stdout).to_string();
    if let Some(i) = s.find("\"notAfter\":\"") {
      let rest = &s[i + 12..];
      if let Some(j) = rest.find('"') { return rest[..j].to_string(); }
    }
  }
  String::new()
}

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        info()
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_object("a", ax);
            result_obj
        }
        Err(err) => {
            let mut err_obj = DataObject::new();
            err_obj.put_string("status", "err");

            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };

            err_obj.put_string("msg", &msg);
            // Wrapped in the same `a` envelope a successful return uses.
            // Unwrapped, callers that unpack the envelope (newbound's
            // format_result, for one) report an opaque 500 — "Not an object:
            // DString(\"err\")" — instead of this message.
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", err_obj);
            result_obj
        }
    }
}

pub fn info() -> DataObject {
let mut jo = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");
let f = build.join("version.txt");

// systemd is optional: containers, macOS and plain-process installs run
// tunnels supervised instead. Report which world this host is in so the
// UI can hide the systemd controls where they can only fail.
let has_systemd = Path::new("/run/systemd/system").exists();
jo.put_boolean("systemd", has_systemd);

// What Run now needs on this OS (#2), and whether this host already has it.
jo.put_object("tun", nb_tun_access(&build.join("nebula"), ""));

// Where Newbound's own web UI listens (#9): a loopback address means phones
// and other members can't open it over the overlay. Read-only - no config writes.
let g0 = DataStore::globals();
let mut http_address = String::new();
if g0.has("system") {
  let system = g0.get_object("system");
  if system.has("config") {
    http_address = system.get_object("config").try_get_string("http_address").unwrap_or_default();
  }
}
if http_address.is_empty() {
  // not booted through the app server (e.g. `newbound exec`): read the file,
  // and fall back to flowlang's own default when the key is absent
  let cp = nbdir.join("config.properties");
  if cp.exists() {
    let p = read_properties(cp.display().to_string());
    if p.has("http_address") { http_address = p.get_string("http_address"); }
  }
  if http_address.is_empty() { http_address = "0.0.0.0".to_string(); }
}
jo.put_string("http_address", &http_address);

// Newbound's own port, for the firewall hint below and for phones
let mut http_port = String::new();
if g0.has("system") {
  let system = g0.get_object("system");
  if system.has("config") {
    let c = system.get_object("config");
    if c.has("http_port") { http_port = ndata::data::Data::as_string(c.get_property("http_port")); }
  }
}
if http_port.is_empty() {
  let cp = nbdir.join("config.properties");
  if cp.exists() {
    let p = read_properties(cp.display().to_string());
    if p.has("http_port") { http_port = p.get_string("http_port"); }
  }
}
jo.put_string("http_port", &http_port);

// Linux + ufw (QA, Phase 5): with ufw on (default inbound DROP), a phone can
// join the tunnel and still not reach Newbound or the lighthouse port. Rules
// can't be read without root, so the page only says ufw is on and shows the
// exact allow rules. Other OS firewalls: not detected (said in the UI).
let mut ufw = false;
if std::env::consts::OS == "linux" {
  if let Ok(s) = fs::read_to_string("/etc/ufw/ufw.conf") {
    if s.lines().any(|l| l.trim() == "ENABLED=yes") {
      let active = std::process::Command::new("systemctl").args(["is-active", "ufw"]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active").unwrap_or(true);
      ufw = active;
    }
  }
}
jo.put_boolean("ufw", ufw);

// networks named in the start= key of botd.properties auto-start at boot
let mut bootlist: Vec<String> = Vec::new();
let bp = root.join("botd.properties");
if bp.exists() {
  let p = read_properties(bp.into_os_string().into_string().unwrap());
  if p.has("start") {
    for s in p.get_string("start").split(",") {
      let s = s.trim();
      if s != "" { bootlist.push(s.to_string()); }
    }
  }
}

// supervised tunnel state lives in this process's globals (start/stop)
let g = DataStore::globals();
let sup = match g.has("nebulaservices") { true => Some(g.get_object("nebulaservices")), false => None };

if f.exists() {
  let p = read_properties(f.into_os_string().into_string().unwrap());
  jo.put_string("tag_name", &p.get_string("version"));
  jo.put_string("binary_name", &p.get_string("binary"));

  let home = root.join("networks");
  let _x = fs::create_dir_all(&home);
  let mut networks = DataArray::new();
  jo.put_array("networks", networks.clone());
  for file in fs::read_dir(&home).unwrap() {
    let f2 = file.unwrap();
    let name = f2.file_name().into_string().unwrap();
    let f2 = f2.path();
    if f2.join("ca.crt").exists() {
      let mut jo2 = DataObject::new();
      networks.push_object(jo2.clone());
      jo2.put_string("name", &name);
// certificate lifetimes (#5): the page shows them and warns 30 days ahead
let certbin = build.join("nebula-cert");
jo2.put_string("ca_not_after", &nb_cert_not_after(&certbin, &f2.join("ca.crt")));
jo2.put_string("host_not_after", &nb_cert_not_after(&certbin, &f2.join("host.crt")));

      let mut service = false;
      let mut service_running = false;
      if has_systemd {
        service = Path::new(&("/etc/systemd/system/".to_string()+&name+".service")).exists();
        if service {
          let mut da = DataArray::new();
          da.push_string("systemctl");
          da.push_string("is-active");
          da.push_string(&name);
          service_running = system_call(da).get_string("out").trim() == "active".to_string();
        }
      }
      jo2.put_boolean("service", service);
      jo2.put_boolean("service_running", service_running);

      let mut supervised = false;
      if let Some(s) = &sup {
        if s.has(&name) { supervised = s.get_object(&name).get_boolean("running"); }
      }
      jo2.put_boolean("supervised", supervised);
      jo2.put_boolean("running", service_running || supervised);
      jo2.put_boolean("boot", bootlist.contains(&name));

      let owner;
      if f2.join("ca.key").exists() { owner = "local".to_string(); }
      else { owner = fs::read_to_string(f2.join("owner.txt")).unwrap().trim().to_owned(); }
      jo2.put_string("owner", &owner);
      let jo3 = DataObject::from_string(&fs::read_to_string(f2.join("connection.json")).unwrap());
      jo2.put_object("config", jo3);
    }
  }
}
else {
  jo.put_string("tag_name", "Not Installed");
  jo.put_string("binary_name", "N/A");
}

jo
}
