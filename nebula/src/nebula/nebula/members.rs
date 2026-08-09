use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;

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
        members(arg_0)
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

pub fn members(servicename: String) -> DataObject {
let mut o = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);

let f = home.join("members");
if f.exists(){
  for file in std::fs::read_dir(&f).unwrap() {
    let path = file.unwrap().path();
    // key by the member's name, not the full path - the UI looks peers up by id
    let name = path.file_name().unwrap().to_str().unwrap().to_string();
    let f2 = path.join("info.json");
    if f2.exists(){
      let s = std::fs::read_to_string(f2).unwrap();
      let jo2 = DataObject::from_string(&s);
      o.put_object(&name, jo2);
    }
  }
}

o

}
