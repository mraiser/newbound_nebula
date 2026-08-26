use ndata::dataobject::DataObject;
use std::process::Command;
use ndata::dataarray::DataArray;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        releases()
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

pub fn releases() -> DataObject {
let mut o = DataObject::new();
let out = Command::new("curl")
  .args(["-fsSL", "--max-time", "20", "https://api.github.com/repos/slackhq/nebula/releases?per_page=15"])
  .output();
match out {
  Err(e) => {
    o.put_string("status", "err");
    o.put_string("msg", &format!("could not run curl: {}", e));
  }
  Ok(res) => {
    if !res.status.success() {
      o.put_string("status", "err");
      o.put_string("msg", &format!("could not reach api.github.com: {}", String::from_utf8_lossy(&res.stderr).trim()));
    } else {
      let text = String::from_utf8_lossy(&res.stdout).to_string();
      let all = DataArray::from_string(&text);
      let mut list = DataArray::new();
      for r in all.objects() {
        let r = r.object();
        if r.has("draft") && r.get_boolean("draft") { continue; }
        let mut e = DataObject::new();
        e.put_string("tag_name", &r.get_string("tag_name"));
        e.put_boolean("prerelease", r.has("prerelease") && r.get_boolean("prerelease"));
        let mut assets = DataArray::new();
        if r.has("assets") {
          for a in r.get_array("assets").objects() {
            let a = a.object();
            let name = a.get_string("name");
            if name.ends_with(".tar.gz") || name.ends_with(".zip") {
              let mut ae = DataObject::new();
              ae.put_string("name", &name);
              ae.put_string("url", &a.get_string("browser_download_url"));
              assets.push_object(ae);
            }
          }
        }
        e.put_array("assets", assets);
        list.push_object(e);
      }
      o.put_string("status", "ok");
      o.put_array("list", list);
    }
  }
}
o
}
