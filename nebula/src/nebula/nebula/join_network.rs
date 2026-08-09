use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use crate::nebula::nebula::save_config::save_config;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "subnet", "ipaddress", "port", "owner", "ca_crt", "host_crt", "host_key", "lighthouses", "groups"] {
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
        let arg_1: String = o.get_string("subnet");
        let arg_2: String = o.get_string("ipaddress");
        let arg_3: String = o.get_string("port");
        let arg_4: String = o.get_string("owner");
        let arg_5: String = o.get_string("ca_crt");
        let arg_6: String = o.get_string("host_crt");
        let arg_7: String = o.get_string("host_key");
        let arg_8: DataObject = o.get_object("lighthouses");
        let arg_9: String = o.get_string("groups");
        join_network(arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6, arg_7, arg_8, arg_9)
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

pub fn join_network(servicename: String, subnet: String, ipaddress: String, port: String, owner: String, ca_crt: String, host_crt: String, host_key: String, lighthouses: DataObject, groups: String) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let _x = std::fs::create_dir_all(&home);

let _x = std::fs::write(&home.join("ca.crt"), &ca_crt).unwrap();
let _x = std::fs::write(&home.join("host.crt"), &host_crt).unwrap();
let _x = std::fs::write(&home.join("host.key"), &host_key).unwrap();
let _x = std::fs::write(&home.join("owner.txt"), &owner).unwrap();

let mut jo = DataObject::new();
jo.put_string("port", &port);
jo.put_string("subnet", &subnet);
jo.put_string("ip_address", &ipaddress);
jo.put_string("host", "0.0.0.0");
jo.put_boolean("am_lighthouse", false);
jo.put_object("lighthouses", lighthouses);
jo.put_boolean("use_yaml", false);
jo.put_string("groups", &groups);

save_config(servicename, jo)
}
