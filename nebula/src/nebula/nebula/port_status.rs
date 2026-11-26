use ndata::dataobject::DataObject;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["port"] {
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
        let arg_0: String = o.get_string("port");
        port_status(arg_0)
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

pub fn port_status(port: String) -> DataObject {
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

}
