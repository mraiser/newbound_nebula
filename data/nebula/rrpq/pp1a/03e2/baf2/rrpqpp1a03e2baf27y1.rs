let mut o = DataObject::new();
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let cj = nbdir.join("runtime").join("nebula").join("networks").join(&servicename).join("connection.json");
if !cj.exists() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("this instance has no network named {}", servicename));
  return o;
}
let mut conn = DataObject::from_string(&fs::read_to_string(cj).unwrap());
conn.put_object("hosts", hosts);
let _saved = save_config(servicename.to_owned(), conn);

// a running supervised tunnel picks the new map up via a quick bounce
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
o.put_string("msg", if restarted { "host map updated, tunnel restarted" } else { "host map updated" });
o