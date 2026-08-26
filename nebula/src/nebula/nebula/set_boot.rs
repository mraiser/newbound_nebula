use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "enabled"] {
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
        let arg_1: bool = o.get_boolean("enabled");
        set_boot(arg_0, arg_1)
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

pub fn set_boot(servicename: String, enabled: bool) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let _x = fs::create_dir_all(&root);
let bp = root.join("botd.properties");

// read the whole file, preserving every key we don't own
let mut lines: Vec<String> = Vec::new();
let mut start: Vec<String> = Vec::new();
let mut seen = false;
if bp.exists() {
  for line in fs::read_to_string(&bp).unwrap().lines() {
    if line.trim_start().starts_with("start=") {
      seen = true;
      for s in line.trim_start()[6..].split(",") {
        let s = s.trim();
        if s != "" { start.push(s.to_string()); }
      }
      lines.push("start=".to_string()); // placeholder, rewritten below
    } else {
      lines.push(line.to_string());
    }
  }
}
if !seen { lines.push("start=".to_string()); }

start.retain(|s| s != &servicename);
if enabled { start.push(servicename.to_owned()); }
let startline = format!("start={}", start.join(","));
let text = lines.iter().map(|l| if l == "start=" { startline.clone() } else { l.clone() })
  .collect::<Vec<String>>().join("\n") + "\n";
fs::write(&bp, &text).unwrap();

let mut o = DataObject::new();
o.put_string("status", "ok");
o.put_string("start", &start.join(","));
o.put_string("msg", if enabled { "will start at boot" } else { "will not start at boot" });
o
}
