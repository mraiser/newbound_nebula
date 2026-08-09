use ndata::dataobject::DataObject;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

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
        uninstall_service(arg_0)
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

pub fn uninstall_service(servicename: String) -> String {
//let store = DataStore::new().root;
//let nbdir = store.parent().unwrap().to_owned();
//let root = nbdir.join("runtime").join("nebula");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("stop");
sa.push_string(&servicename);
let res = system_call(sa);
let s = res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("disable");
sa.push_string(&servicename);
let res = system_call(sa);
let s = s + &res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("rm");
sa.push_string(&("/etc/systemd/system/".to_string()+&servicename+".service"));
let res = system_call(sa);
let mut s = s + &res.get_string("out") + &res.get_string("err");

if s == "" { s = "OK".to_string(); }

s
}
