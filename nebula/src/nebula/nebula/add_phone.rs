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

/// Standard base64 (with or without padding) -> bytes; None on bad input.
fn nb_b64_decode(s: &str) -> Option<Vec<u8>> {
  let mut out = Vec::new(); let mut buf = 0u32; let mut bits = 0;
  for c in s.chars() {
    let v = match c { 'A'..='Z' => c as u32 - 65, 'a'..='z' => c as u32 - 71, '0'..='9' => c as u32 + 4, '+' => 62, '/' => 63,
                      '=' => break, ' ' | '\n' | '\r' | '\t' => continue, _ => return None };
    buf = (buf << 6) | v; bits += 6;
    if bits >= 8 { bits -= 8; out.push((buf >> bits) as u8); buf &= (1 << bits) - 1; }
  }
  Some(out)
}

/// SHA-256 (FIPS 180-4), hex. Small and dependency-free on purpose.
fn nb_sha256_hex(data: &[u8]) -> String {
  const K: [u32; 64] = [0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
  let mut h: [u32; 8] = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
  let mut m = data.to_vec(); let bitlen = (data.len() as u64) * 8;
  m.push(0x80); while m.len() % 64 != 56 { m.push(0); } m.extend_from_slice(&bitlen.to_be_bytes());
  for chunk in m.chunks(64) {
    let mut w = [0u32; 64];
    for i in 0..16 { w[i] = u32::from_be_bytes([chunk[4*i], chunk[4*i+1], chunk[4*i+2], chunk[4*i+3]]); }
    for i in 16..64 {
      let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
      let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
      w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
    }
    let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) = (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
    for i in 0..64 {
      let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
      let ch = (e & f) ^ (!e & g);
      let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
      let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
      let maj = (a & b) ^ (a & c) ^ (b & c);
      let t2 = s0.wrapping_add(maj);
      hh = g; g = f; f = e; e = d.wrapping_add(t1); d = c; c = b; b = a; a = t1.wrapping_add(t2);
    }
    h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b); h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f); h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
  }
  h.iter().map(|x| format!("{:08x}", x)).collect()
}

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "device_name", "groups", "pubkey"] {
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
        let arg_3: String = o.get_string("pubkey");
        add_phone(arg_0, arg_1, arg_2, arg_3)
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

pub fn add_phone(servicename: String, device_name: String, groups: String, pubkey: String) -> DataObject {
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
// Mode B2 (ENROLLMENT §3.2): the phone made its own keypair and sent only
// its public key. Check it before anything is recorded.
let pubkey = pubkey.trim().replace("\r", "");
let b2 = !pubkey.is_empty();
let mut pubkey_fingerprint = String::new();
if b2 {
  let begin = "-----BEGIN NEBULA X25519 PUBLIC KEY-----";
  let end = "-----END NEBULA X25519 PUBLIC KEY-----";
  let (bi, ei) = match (pubkey.find(begin), pubkey.find(end)) { (Some(b), Some(e)) if e > b => (b, e), _ =>
    return nb_err("That isn't a nebula public key. Paste the whole block, from -----BEGIN NEBULA X25519 PUBLIC KEY----- to the END line.", "bad_pubkey") };
  let raw = match nb_b64_decode(&pubkey[bi + begin.len()..ei]) { Some(r) if r.len() == 32 => r, _ =>
    return nb_err("That public key is damaged (it should hold 32 bytes). Copy it again from the phone.", "bad_pubkey") };
  pubkey_fingerprint = nb_sha256_hex(&raw);
}
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
let mut args: Vec<String> = vec!["sign".into(), "-ca-crt".into(), home.join("ca.crt").display().to_string(),
  "-ca-key".into(), home.join("ca.key").display().to_string(), "-name".into(), device_name.clone(), "-ip".into(), ip_cidr.clone(),
  "-groups".into(), gl.join(","), "-out-crt".into(), tmp.join("host.crt").display().to_string()];
if b2 {
  // sign the phone's own public key: no private key is made here at all
  let _ = fs::write(tmp.join("host.pub"), format!("{}\n", pubkey));
  args.push("-in-pub".into()); args.push(tmp.join("host.pub").display().to_string());
} else {
  args.push("-out-key".into()); args.push(tmp.join("host.key").display().to_string());
}
let out = std::process::Command::new(&certbin).args(&args).output();
let key = fs::read_to_string(tmp.join("host.key")).unwrap_or_default();
let crt = fs::read_to_string(tmp.join("host.crt")).unwrap_or_default();
let _ = fs::remove_dir_all(&tmp);
let ok = match &out { Ok(o) => o.status.success(), Err(_) => false };
if !ok || (!b2 && key.is_empty()) || crt.is_empty() {
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
info.put_string("mode", if b2 { "b2" } else { "a" });
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
j.put_string("kind", if b2 { "cert" } else { "join" });
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
// Mode A carries the key (secret); B2's cert code never does, and says
// which public key it was signed for instead.
if b2 { j.put_string("pubkey_fingerprint", &pubkey_fingerprint); } else { j.put_string("key", &key); }
j.put_int("expires", created + 600);

let mut o = DataObject::new();
o.put_string("status", "ok");
o.put_string("member_id", &id);
o.put_object("join", j);
o

}
