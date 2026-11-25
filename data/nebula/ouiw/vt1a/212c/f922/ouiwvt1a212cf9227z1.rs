// Is this UDP port free on this machine (#8)? Binding is the only portable
// test: it works the same on Linux, macOS, Windows and the Pi.
let mut o = DataObject::new();
o.put_string("status", "ok");
o.put_string("port", &port);
let p: u16 = match port.trim().parse() {
  Ok(p) if p > 0 => p,
  _ => {
    o.put_string("status", "err");
    o.put_string("msg", "A port is a number from 1 to 65535.");
    return o;
  }
};
match std::net::UdpSocket::bind(("0.0.0.0", p)) {
  Ok(_) => { o.put_boolean("bound", false); }
  Err(e) => {
    o.put_boolean("bound", true);
    o.put_string("why", &e.to_string());
  }
}
o
