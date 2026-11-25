// Remove a member (ENROLLMENT §4): its cert fingerprint goes on the
// network's blocklist, so this host and - once the owner's page pushes the
// list - every Newbound member refuses it, then its folder is deleted.
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let net = root.join("networks").join(&servicename);
let home = net.join("members").join(&peer);
let mut o = DataObject::new();
if !home.exists() {
  o.put_string("status", "err");
  o.put_string("msg", "No such member");
  return o;
}
let mut name = peer.clone();
let mut fingerprint = String::new();
if let Ok(s) = fs::read_to_string(home.join("info.json")) {
  let info = DataObject::from_string(&s);
  if info.has("name") { name = info.get_string("name"); }
  if info.has("fingerprint") { fingerprint = info.get_string("fingerprint"); }
}
if fingerprint.is_empty() && home.join("host.crt").exists() {
  // members added before Phase 3 have no fingerprint on record: read the cert
  if let Ok(out) = std::process::Command::new(root.join("bin").join("nebula-cert"))
      .args(["print", "-json", "-path", &home.join("host.crt").display().to_string()]).output() {
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if let Some(i) = s.find("\"fingerprint\":\"") {
      let rest = &s[i + 15..];
      if let Some(j) = rest.find('"') { fingerprint = rest[..j].to_string(); }
    }
  }
}

let mut blocklist = DataObject::new();
if let Ok(s) = fs::read_to_string(net.join("connection.json")) {
  let conn = DataObject::from_string(&s);
  if conn.has("blocklist") { blocklist = conn.get_object("blocklist"); }
}
if !fingerprint.is_empty() {
  let mut e = DataObject::new();
  e.put_string("name", &name);
  e.put_int("removed", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64);
  blocklist.put_object(&fingerprint, e);
}
let _x = fs::remove_dir_all(&home);

let applied = update_blocklist(servicename.to_owned(), blocklist.clone());
o.put_string("status", "ok");
o.put_string("name", &name);
o.put_string("fingerprint", &fingerprint);
o.put_object("blocklist", blocklist);
o.put_boolean("restarted", applied.has("restarted") && applied.get_boolean("restarted"));
o.put_string("msg", &if fingerprint.is_empty() {
  format!("{} removed, but its certificate couldn't be read, so it isn't blocked. Rotate the CA (recreate the network) to force it out.", name)
} else if applied.has("raw_yaml") && applied.get_boolean("raw_yaml") {
  format!("{} removed. This network uses raw YAML, so add its fingerprint under pki.blocklist yourself.", name)
} else if applied.has("restarted") && applied.get_boolean("restarted") {
  format!("{} removed and its certificate blocked (the tunnel restarted to apply it).", name)
} else { format!("{} removed and its certificate blocked.", name) });
o
