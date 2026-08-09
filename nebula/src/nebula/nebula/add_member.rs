use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "peer", "ipaddress", "groups"] {
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
        let arg_1: String = o.get_string("peer");
        let arg_2: String = o.get_string("ipaddress");
        let arg_3: String = o.get_string("groups");
        add_member(arg_0, arg_1, arg_2, arg_3)
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

pub fn add_member(servicename: String, peer: String, ipaddress: String, groups: String) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let homestr = home.canonicalize().unwrap().into_os_string().into_string().unwrap();
let bin = root.join("bin");

let memberhome = home.join("members").join(&peer);
let _x = fs::create_dir_all(&memberhome);
let memberstr = memberhome.canonicalize().unwrap().into_os_string().into_string().unwrap();

let mut da = DataArray::new();
da.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
da.push_string("sign");
da.push_string("-ca-crt");
da.push_string(&(homestr.to_owned()+"/ca.crt"));
da.push_string("-ca-key");
da.push_string(&(homestr.to_owned()+"/ca.key"));
da.push_string("-name");
da.push_string(&peer);
da.push_string("-ip");
da.push_string(&ipaddress);
da.push_string("-out-crt");
da.push_string(&(memberstr.to_owned()+"/host.crt"));
da.push_string("-out-key");
da.push_string(&(memberstr.to_owned()+"/host.key"));
if groups != "".to_string() {
  da.push_string("-groups");
  da.push_string(&groups);
}
let res = system_call(da);
let s = res.get_string("out") + &res.get_string("err");
if s != "".to_string() {
  // don't leave an empty member folder behind on a failed signing
  let _x = fs::remove_dir_all(&memberhome);
  let mut o = DataObject::new();
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

let mut jo = DataObject::new();
jo.put_string("ip_address", &ipaddress);
jo.put_string("groups", &groups);
let _x = fs::write(&memberhome.join("info.json"), &jo.to_string()).unwrap();

jo.put_string("ca_crt", &fs::read_to_string(home.join("ca.crt")).unwrap());
jo.put_string("host_crt", &fs::read_to_string(memberhome.join("host.crt")).unwrap());
jo.put_string("host_key", &fs::read_to_string(memberhome.join("host.key")).unwrap());

jo

}
