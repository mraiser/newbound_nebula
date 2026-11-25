use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;
use crate::nebula::nebula::save_config::save_config;
use crate::nebula::nebula::start::start;
use crate::nebula::nebula::stop::stop;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "mode"] {
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
        let arg_1: String = o.get_string("mode");
        set_firewall(arg_0, arg_1)
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

pub fn set_firewall(servicename: String, mode: String) -> DataObject {
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

}
