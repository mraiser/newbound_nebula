let mut o = DataObject::new();
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let cj = nbdir.join("runtime").join("nebula").join("networks").join(&servicename).join("connection.json");
if !cj.exists() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("this instance has no network named {}", servicename));
  return o;
}
let conn = DataObject::from_string(&fs::read_to_string(cj).unwrap());
let port = conn.get_string("port");

// Candidate endpoints for THIS host's nebula socket: the primary outbound
// interface (a UDP connect sends nothing - it just resolves routing).
let mut list = DataArray::new();
if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
  if sock.connect("8.8.8.8:80").is_ok() {
    if let Ok(a) = sock.local_addr() {
      list.push_string(&format!("{}:{}", a.ip(), port));
    }
  }
}

// What this instance has OBSERVED for the asked-about peer: the source
// address the platform recorded on that peer's user record when its
// encrypted peer connection was made. That is the peer's public face from
// here - exactly what a lighthouse would have told us.
let mut observed = "".to_string();
if observe != "" {
  let users = crate::API.security.security.users();
  if users.has(&observe) {
    let u = users.get_object(&observe);
    if u.has("address") { observed = u.get_string("address"); }
  }
}
o.put_string("status", "ok");
o.put_array("endpoints", list);
o.put_string("port", &port);
o.put_string("observed", &observed);
o