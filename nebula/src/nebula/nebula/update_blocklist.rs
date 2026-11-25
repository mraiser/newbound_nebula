use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;
use crate::nebula::nebula::save_config::save_config;
use crate::nebula::nebula::start::start;
use crate::nebula::nebula::stop::stop;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "blocklist"] {
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
        let arg_1: DataObject = o.get_object("blocklist");
        update_blocklist(arg_0, arg_1)
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

pub fn update_blocklist(servicename: String, blocklist: DataObject) -> DataObject {
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

}
