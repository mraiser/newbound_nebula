use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::fs;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename", "config"] {
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
        let arg_1: DataObject = o.get_object("config");
        save_config(arg_0, arg_1)
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

pub fn save_config(servicename: String, config: DataObject) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let _x = fs::create_dir_all(&home);

let mut config = config;
let f2 = home.join("config.yml");
if config.get_boolean("use_yaml") {
  let _x = fs::write(f2, &config.get_string("yaml")).unwrap();
}
else {
  let mut shm = "".to_string();
  let mut hosts = "".to_string();
  let lhs = config.get_object("lighthouses");
  for peer in lhs.clone().keys() {
    let lh = lhs.get_object(&peer);
    shm = shm + "  \""+(&lh.get_string("private_ip"))+"\": [\""+(&lh.get_string("public_ip"))+":"+(&lh.get_string("port"))+"\"]\n";
    hosts = hosts + "    - \""+(&lh.get_string("private_ip"))+"\"\n";
  }

  // The one home of the config.yml template - build_config renders through here.
  let path = &home.canonicalize().unwrap().into_os_string().into_string().unwrap();
  let s = "pki:\n  ca: ".to_string()+path+"/ca.crt\n  cert: "+path+"/host.crt\n  key: "+path+"/host.key\nstatic_host_map:\n"+(&shm)+"lighthouse:\n  am_lighthouse: "+(&config.get_boolean("am_lighthouse").to_string())+"\n  interval: 60\n  hosts:\n"+(&hosts)+"listen:\n  host: "+(&config.get_string("host"))+"\n  port: "+(&config.get_string("port"))+"\npunchy: true\ntun:\n  dev: "+(&servicename)+"\n  drop_local_broadcast: false\n  drop_multicast: false\n  tx_queue: 500\n  mtu: 1300\n  routes:\n  unsafe_routes:\nlogging:\n  level: info\n  format: text\nfirewall:\n  conntrack:\n    tcp_timeout: 120h\n    udp_timeout: 3m\n    default_timeout: 10m\n    max_connections: 100000\n  outbound:\n    - port: any\n      proto: any\n      host: any\n  inbound:\n    - port: any\n      proto: icmp\n      host: any\n    - port: any\n      proto: tcp\n      host: any\n";
  let _x = fs::write(&f2, &s).unwrap();
  config.put_string("yaml", &s);

  let f2 = home.join("connection.json");
  let _x = fs::write(f2, &config.to_string()).unwrap();
}

config

}
