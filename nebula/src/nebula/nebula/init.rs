use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use crate::nebula::nebula::start::start;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        init()
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

pub fn init() -> DataObject {
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
}
