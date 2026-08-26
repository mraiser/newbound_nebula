use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use ndata::dataarray::DataArray;
use std::fs;
use std::net::UdpSocket;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "observe"] {
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
        let arg_1: String = o.get_string("observe");
        endpoints(arg_0, arg_1)
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

pub fn endpoints(servicename: String, observe: String) -> DataObject {
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
}
