use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;
use flowlang::flowlang::file::write_properties::write_properties;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["url", "version"] {
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
        let arg_0: String = o.get_string("url");
        let arg_1: String = o.get_string("version");
        install_release(arg_0, arg_1)
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

pub fn install_release(url: String, version: String) -> String {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");
let _x = std::fs::create_dir_all(&build);

let f = build.join("download.tgz");
let fstr = f.to_owned().into_os_string().into_string().unwrap();

// no http-client dependency: curl over system_call, like the platform's
// own git/zip/unzip/tar calls
let mut sa = DataArray::new();
sa.push_string("curl");
sa.push_string("-fsSL");
sa.push_string("-o");
sa.push_string(&fstr);
sa.push_string(&url);
let res = system_call(sa);
let err = res.get_string("err");
if !f.exists() || std::fs::metadata(&f).unwrap().len() == 0 {
  let _x = std::fs::remove_file(&f);
  return "ERROR: Download failed: ".to_string()+&err;
}

let mut sa = DataArray::new();
sa.push_string("tar");
sa.push_string("-xzf");
sa.push_string(&fstr);
sa.push_string("-C");
sa.push_string(&build.to_owned().into_os_string().into_string().unwrap());
let res = system_call(sa);
let err = res.get_string("err");
let _x = std::fs::remove_file(&f);
if err != "".to_string() { return "ERROR: ".to_string()+&err; }

let pos = url.rfind('/').unwrap();
let binary = &url[pos+1..];

let mut p = DataObject::new();
p.put_string("version", &version);
p.put_string("binary", &binary);
write_properties(build.join("version.txt").into_os_string().into_string().unwrap(), p);

"OK".to_string()

}
