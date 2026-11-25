// Firewall mode of a network (ENROLLMENT §5): "member" makes the default
// inbound rules accept only certs in group `member`; "" keeps `host: any`.
// The owner calls it on a new member right after join_network, and again
// on Sync mesh, so members follow the owner's network.
let mut o = DataObject::new();
if mode != "" && mode != "member" {
  o.put_string("status", "err");
  o.put_string("msg", "firewall mode is \"member\" or empty");
  return o;
}
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let cj = nbdir.join("runtime").join("nebula").join("networks").join(&servicename).join("connection.json");
if !cj.exists() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("this instance has no network named {}", servicename));
  return o;
}
let mut conn = DataObject::from_string(&fs::read_to_string(&cj).unwrap());
let was = if conn.has("firewall") { conn.get_string("firewall") } else { String::new() };
o.put_string("status", "ok");
if was == mode { o.put_string("msg", "unchanged"); o.put_boolean("changed", false); return o; }
conn.put_string("firewall", &mode);
let _saved = save_config(servicename.to_owned(), conn);
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
o.put_boolean("changed", true);
o.put_boolean("restarted", restarted);
o.put_string("msg", "firewall mode set");
o
