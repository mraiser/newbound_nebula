use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;
use crate::nebula::nebula::save_config::save_config;

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
        build_config(arg_0)
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_string("a", &ax);
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

pub fn build_config(servicename: String) -> String {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let f = root.join("networks").join(&servicename).join("connection.json");
let jo = DataObject::from_string(&fs::read_to_string(f).unwrap());

// one template: save_config renders (and re-persists) the YAML
let jo = save_config(servicename, jo);

jo.get_string("yaml")

}
