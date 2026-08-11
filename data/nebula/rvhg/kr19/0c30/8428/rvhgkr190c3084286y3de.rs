let g = DataStore::globals().get_object("system").get_object("apps").get_object("nebula").get_object("runtime");
if g.has("start") {
  let s = g.get_string("start");
  let s = s.split(",");
  for servicename in s {
    println!("STARTING NEBULA SERVICE {}", &servicename);
    start(servicename.to_string());
  }
}
g