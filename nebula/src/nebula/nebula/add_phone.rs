use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use ndata::dataarray::DataArray;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// One string field from `nebula-cert print -json` (first occurrence); "" if unknown.
fn nb_cert_json_str(certbin: &std::path::Path, crt: &std::path::Path, key: &str) -> String {
  let out = std::process::Command::new(certbin).args(["print", "-json", "-path", &crt.display().to_string()]).output();
  if let Ok(o) = out {
    let s = String::from_utf8_lossy(&o.stdout).to_string();
    let pat = format!("\"{}\":\"", key);
    if let Some(i) = s.find(&pat) {
      let rest = &s[i + pat.len()..];
      if let Some(j) = rest.find('"') { return rest[..j].to_string(); }
    }
  }
  String::new()
}

fn nb_err(msg: &str, reason: &str) -> DataObject {
  let mut o = DataObject::new();
  o.put_string("status", "err");
  o.put_string("msg", msg);
  if !reason.is_empty() { o.put_string("reason", reason); }
  o
}

/// "host:port" for a lighthouse address; adds the network's port when missing.
fn nb_with_port(addr: &str, port: &str) -> String {
  let a = addr.trim();
  if a.is_empty() { return String::new(); }
  if a.starts_with('[') { return if a.contains("]:") { a.to_string() } else { format!("{}:{}", a, port) }; }
  if a.matches(':').count() == 1 { a.to_string() }
  else if a.contains(':') { format!("[{}]:{}", a, port) }     // bare IPv6
  else { format!("{}:{}", a, port) }
}

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "device_name", "groups"] {
        if !o.has(p) {
            let mut e = DataObject::new();
            e.put_string("status", "err");
            e.put_string("msg", &format!("missing required parameter: {}", p));
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", e);
            return result_obj;
        }
    }
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let arg_0: String = o.get_string("servicename");
        let arg_1: String = o.get_string("device_name");
        let arg_2: String = o.get_string("groups");
        add_phone(arg_0, arg_1, arg_2)
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

pub fn add_phone(servicename: String, device_name: String, groups: String) -> DataObject {
// Mode A of docs ENROLLMENT.md (§2): Newbound makes the phone's keypair and
// hands everything to the phone in one join code. The private key is read
// into memory and returned once; it is never kept on this instance (only
// host.crt + info.json go under members/).
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let certbin = root.join("bin").join("nebula-cert");

let device_name = device_name.trim().to_string();
if device_name.is_empty() { return nb_err("Give the phone a name first.", "bad_name"); }
if device_name.chars().count() > 64 { return nb_err("Keep the phone's name under 64 characters.", "bad_name"); }
if !home.join("ca.key").exists() { return nb_err("Only the instance that owns this network can add phones.", "not_owner"); }
if !certbin.exists() { return nb_err("Install the nebula engine first.", "no_engine"); }

let cfgpath = home.join("connection.json");
let cfg = match fs::read_to_string(&cfgpath) { Ok(s) => DataObject::from_string(&s), Err(_) => return nb_err("This network has no saved configuration yet.", "no_config") };
let port = cfg.get_string("port");
let myip = cfg.get_string("ip_address");
let myplain = myip.split('/').next().unwrap_or("").to_string();

// Lighthouses (phones aren't on the peer mesh, so they need at least one with
// a public address): this host when it is one and knows its public address,
// plus every lighthouse entry that has a public IP.
let mut lighthouses = DataArray::new();
if cfg.has("am_lighthouse") && cfg.get_boolean("am_lighthouse") {
  let pa = if cfg.has("public_address") { cfg.get_string("public_address") } else { String::new() };
  let a = nb_with_port(&pa, &port);
  if !a.is_empty() {
    let mut lh = DataObject::new();
    lh.put_string("nebula_ip", &myplain);
    let mut addrs = DataArray::new(); addrs.push_string(&a);
    lh.put_array("addrs", addrs);
    lighthouses.push_object(lh);
  }
}
let mut used: Vec<String> = vec![myplain.clone()];
if cfg.has("lighthouses") {
  let lhs = cfg.get_object("lighthouses");
  for id in lhs.clone().keys() {
    let l = lhs.get_object(&id);
    let pip = l.get_string("private_ip").split('/').next().unwrap_or("").to_string();
    used.push(pip.clone());
    let pub_ip = l.get_string("public_ip");
    let lport = if l.has("port") { l.get_string("port") } else { port.clone() };
    let a = nb_with_port(&pub_ip, &lport);
    if pip.is_empty() || a.is_empty() || pip == myplain { continue; }
    let mut lh = DataObject::new();
    lh.put_string("nebula_ip", &pip);
    let mut addrs = DataArray::new(); addrs.push_string(&a);
    lh.put_array("addrs", addrs);
    lighthouses.push_object(lh);
  }
}
if lighthouses.len() == 0 {
  let selfmsg = if cfg.has("am_lighthouse") && cfg.get_boolean("am_lighthouse") {
    "Add a lighthouse first \u{2014} phones need one. This host is marked as a lighthouse: enter its public address on the Service tab."
  } else { "Add a lighthouse first \u{2014} phones need one." };
  return nb_err(selfmsg, "no_lighthouse");
}

// Next free overlay address: skip this host, lighthouses and every member.
let members = home.join("members");
if let Ok(rd) = fs::read_dir(&members) {
  for e in rd.flatten() {
    if let Ok(s) = fs::read_to_string(e.path().join("info.json")) {
      let jo = DataObject::from_string(&s);
      if jo.has("ip_address") { used.push(jo.get_string("ip_address").split('/').next().unwrap_or("").to_string()); }
    }
  }
}
let subnet = cfg.get_string("subnet");
if !subnet.contains('X') || !subnet.contains('/') { return nb_err("This network's subnet isn't in the 192.168.100.X/24 form, so the app can't pick an address.", "bad_subnet"); }
let mask = subnet.split('/').nth(1).unwrap_or("24").to_string();
let mut ip = String::new();
for n in 2..255 {
  let cand = subnet.split('/').next().unwrap().replace('X', &n.to_string());
  if !used.contains(&cand) { ip = cand; break; }
}
if ip.is_empty() { return nb_err("This network has no free addresses left.", "full"); }
let ip_cidr = format!("{}/{}", ip, mask);

// Groups: member,phone first (ENROLLMENT §5), then any extras.
let mut gl: Vec<String> = vec!["member".to_string(), "phone".to_string()];
for g in groups.split(',') {
  let g = g.trim();
  if g.is_empty() { continue; }
  if !g.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') { return nb_err("Groups are letters, digits, - and _, separated by commas.", "bad_groups"); }
  if !gl.contains(&g.to_string()) { gl.push(g.to_string()); }
}

// A folder id for the member: slug of the name + a short unique suffix.
let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
let created = now.as_secs() as i64;
let slug: String = device_name.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
let slug = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
let slug = if slug.is_empty() { "phone".to_string() } else { slug.chars().take(32).collect() };
let id = format!("phone-{}-{:06x}", slug, (now.as_nanos() & 0xffffff) as u64);

// Sign in a private scratch folder inside the network folder (the same trust
// boundary as ca.key), read the key, delete the folder at once.
let tmp = home.join(format!(".add-phone-{}", id));
let _ = fs::remove_dir_all(&tmp);
if let Err(e) = fs::create_dir_all(&tmp) { return nb_err(&format!("Couldn't create a temporary folder: {}", e), "io"); }
#[cfg(unix)]
{ use std::os::unix::fs::PermissionsExt; let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o700)); }
let out = std::process::Command::new(&certbin)
  .args(["sign", "-ca-crt", &home.join("ca.crt").display().to_string(), "-ca-key", &home.join("ca.key").display().to_string(),
         "-name", &device_name, "-ip", &ip_cidr, "-groups", &gl.join(","),
         "-out-crt", &tmp.join("host.crt").display().to_string(), "-out-key", &tmp.join("host.key").display().to_string()])
  .output();
let key = fs::read_to_string(tmp.join("host.key")).unwrap_or_default();
let crt = fs::read_to_string(tmp.join("host.crt")).unwrap_or_default();
let _ = fs::remove_dir_all(&tmp);
let ok = match &out { Ok(o) => o.status.success(), Err(_) => false };
if !ok || key.is_empty() || crt.is_empty() {
  let why = match out { Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(), Err(e) => e.to_string() };
  return nb_err(&format!("nebula-cert couldn't sign the phone's certificate: {}", why), "sign_failed");
}

// Record the member: certificate + info only.
let mhome = members.join(&id);
if let Err(e) = fs::create_dir_all(&mhome) { return nb_err(&format!("Couldn't record the phone: {}", e), "io"); }
let _ = fs::write(mhome.join("host.crt"), &crt);
let not_after = nb_cert_json_str(&certbin, &mhome.join("host.crt"), "notAfter");
let fingerprint = nb_cert_json_str(&certbin, &mhome.join("host.crt"), "fingerprint");
let mut info = DataObject::new();
info.put_string("kind", "phone");
info.put_string("name", &device_name);
info.put_string("ip_address", &ip_cidr);
info.put_string("groups", &gl.join(","));
info.put_int("added", created);
info.put_string("not_after", &not_after);
info.put_string("fingerprint", &fingerprint);
let _ = fs::write(mhome.join("info.json"), info.to_string());

// Owner identity and Newbound's own port, from the running platform.
let g = DataStore::globals();
let mut owner_name = String::new();
let mut owner_id = String::new();
let mut http_port = String::new();
if g.has("system") {
  let system = g.get_object("system");
  if system.has("config") {
    let c = system.get_object("config");
    owner_name = c.try_get_string("machineid").unwrap_or_default();
    if c.has("http_port") { http_port = ndata::data::Data::as_string(c.get_property("http_port")); }
  }
  if system.has("apps") {
    let apps = system.get_object("apps");
    if apps.has("app") {
      let app = apps.get_object("app");
      if app.has("runtime") { owner_id = app.get_object("runtime").try_get_string("uuid").unwrap_or_default(); }
    }
  }
}

// Not booted through the app server (e.g. `newbound exec`): read the files.
if owner_name.is_empty() || http_port.is_empty() {
  let cp = nbdir.join("config.properties");
  if let Ok(s) = fs::read_to_string(&cp) {
    for line in s.lines() {
      if let Some(v) = line.strip_prefix("machineid=") { if owner_name.is_empty() { owner_name = v.trim().to_string(); } }
      if let Some(v) = line.strip_prefix("http_port=") { if http_port.is_empty() { http_port = v.trim().to_string(); } }
    }
  }
}
if owner_id.is_empty() {
  if let Ok(s) = fs::read_to_string(nbdir.join("runtime").join("app").join("botd.properties")) {
    for line in s.lines() { if let Some(v) = line.strip_prefix("uuid=") { owner_id = v.trim().to_string(); } }
  }
}

// The Mode A payload, field for field as ENROLLMENT.md §1.2 + §2.
let mut j = DataObject::new();
j.put_int("v", 1);
j.put_string("kind", "join");
j.put_string("network", &servicename);
j.put_string("ca_fingerprint", &nb_cert_json_str(&certbin, &home.join("ca.crt"), "fingerprint"));
let mut owner = DataObject::new();
owner.put_string("name", &owner_name);
owner.put_string("peer_id", &owner_id);
j.put_object("owner", owner);
j.put_string("newbound_url", &if http_port.is_empty() { String::new() } else { format!("http://{}:{}", myplain, http_port) });
j.put_array("lighthouses", lighthouses);
// static_hosts: the mesh map the members already use (their LAN/observed
// endpoints), so a phone on the same Wi-Fi can go direct.
let mut sh = DataObject::new();
if cfg.has("hosts") {
  let hm = cfg.get_object("hosts");
  for hip in hm.clone().keys() {
    let plain = hip.split('/').next().unwrap_or("").to_string();
    let arr = hm.get_array(&hip);
    if arr.len() > 0 { sh.put_array(&plain, arr); }
  }
}
if sh.clone().keys().len() > 0 { j.put_object("static_hosts", sh); }
j.put_int("created", created);
j.put_string("device_name", &device_name);
j.put_string("ip", &ip_cidr);
let mut ga = DataArray::new();
for g in &gl { ga.push_string(g); }
j.put_array("groups", ga);
j.put_string("ca", &fs::read_to_string(home.join("ca.crt")).unwrap_or_default());
j.put_string("cert", &crt);
j.put_string("key", &key);
j.put_int("expires", created + 600);

let mut o = DataObject::new();
o.put_string("status", "ok");
o.put_string("member_id", &id);
o.put_object("join", j);
o

}
