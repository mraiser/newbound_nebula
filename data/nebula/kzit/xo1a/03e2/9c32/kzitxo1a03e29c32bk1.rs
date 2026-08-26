let mut o = DataObject::new();
let mut was = false;
{
  let mut kids = NEBULA_CHILDREN.lock().unwrap();
  if let Some(mut child) = kids.remove(&servicename) {
    was = match child.try_wait() { Ok(None) => true, _ => false };
    let _x = child.kill();
    let _x = child.wait();
  }
}
let mut g = DataStore::globals();
if g.has("nebulaservices") {
  let mut g = g.get_object("nebulaservices");
  if g.has(&servicename) { g.get_object(&servicename).put_boolean("running", false); }
}
o.put_string("status", "ok");
if was {
  println!("STOPPED nebula network {}", &servicename);
  o.put_string("msg", &format!("{} stopped", servicename));
} else {
  o.put_string("msg", &format!("{} was not running as a supervised process", servicename));
}
o.put_boolean("was_running", was);
o