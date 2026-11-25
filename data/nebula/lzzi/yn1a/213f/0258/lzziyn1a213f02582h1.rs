// Removed members (ENROLLMENT §4): replace this network's blocklist (cert
// fingerprint -> {name, removed}) and re-render config.yml. The owner calls
// it on itself and on every Newbound member (Remove, Sync mesh); a running
// supervised tunnel is bounced so the list applies at once.
let mut o = DataObject::new();
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let cj = nbdir.join("runtime").join("nebula").join("networks").join(&servicename).join("connection.json");
if !cj.exists() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("this instance has no network named {}", servicename));
  return o;
}
for fp in blocklist.clone().keys() {
  if fp.len() < 32 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
    o.put_string("status", "err");
    o.put_string("msg", &format!("not a certificate fingerprint: {}", fp));
    return o;
  }
}
let mut conn = DataObject::from_string(&fs::read_to_string(&cj).unwrap());
let raw_yaml = conn.has("use_yaml") && conn.get_boolean("use_yaml");
conn.put_object("blocklist", blocklist);
let _saved = save_config(servicename.to_owned(), conn.clone());
if raw_yaml {
  // save_config writes only the raw YAML in this mode: keep the list on record
  let _ = fs::write(&cj, conn.to_string());
}

let mut restarted = false;
let g = DataStore::globals();
if g.has("nebulaservices") {
  let s = g.get_object("nebulaservices");
  if s.has(&servicename) && s.get_object(&servicename).get_boolean("running") {
    let _x = stop(servicename.to_owned());
    let r = start(servicename.to_owned());
    restarted = r.get_string("status") == "ok";
  }
}
o.put_string("status", "ok");
o.put_boolean("restarted", restarted);
o.put_boolean("raw_yaml", raw_yaml);
o.put_string("msg", &if raw_yaml { "blocklist recorded, but this network uses raw YAML: add the fingerprints under pki.blocklist yourself".to_string() }
  else if restarted { "blocklist updated, tunnel restarted".to_string() } else { "blocklist updated".to_string() });
o
