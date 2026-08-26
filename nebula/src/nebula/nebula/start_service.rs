use ndata::dataobject::DataObject;
use std::process::Command;
use std::path::Path;

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
        start_service(arg_0)
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

pub fn start_service(servicename: String) -> String {
// Run one command, verdict from the exit status; stderr only matters on failure.
fn run(args: &[&str]) -> Result<String, String> {
  let out = Command::new(args[0]).args(&args[1..]).output();
  match out {
    Err(e) => Err(format!("could not run {}: {}", args[0], e)),
    Ok(o) => {
      let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
      if o.status.success() { Ok(text) } else { Err(text.trim().to_string()) }
    }
  }
}

if !Path::new("/run/systemd/system").exists() { return "ERROR: this host does not run systemd - use the supervised Run control instead.".to_string(); }
match run(&["sudo", "systemctl", "start", &servicename]) {
  Ok(_) => "OK".to_string(),
  Err(e) => format!("ERROR: start failed: {}", e),
}
}
