use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use crate::nebula::nebula::start::NEBULA_CHILDREN;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename"] {
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
        stop(arg_0)
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

pub fn stop(servicename: String) -> DataObject {
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
}
