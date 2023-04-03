use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use std::fs;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let a1 = o.get_object("config");
let ax = save_config(a0, a1);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn save_config(servicename:String, config:DataObject) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let _x = fs::create_dir_all(&home);

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
    
  let path = &home.canonicalize().unwrap().into_os_string().into_string().unwrap();
  let s = "pki:\n  ca: ".to_string()+path+"/ca.crt\n  cert: "+path+"/host.crt\n  key: "+path+"/host.key\nstatic_host_map:\n"+(&shm)+"lighthouse:\n  am_lighthouse: "+(&config.get_boolean("am_lighthouse").to_string())+"\n  interval: 60\n  hosts:\n"+(&hosts)+"listen:\n  host: "+(&config.get_string("host"))+"\n  port: "+(&config.get_string("port"))+"\npunchy: true\ntun:\n  dev: "+(&servicename)+"\n  drop_local_broadcast: false\n  drop_multicast: false\n  tx_queue: 500\n  mtu: 1300\n  routes:\n  unsafe_routes:\nlogging:\n  level: info\n  format: text\nfirewall:\n  conntrack:\n    tcp_timeout: 120h\n    udp_timeout: 3m\n    default_timeout: 10m\n    max_connections: 100000\n  outbound:\n    - port: any\n      proto: any\n      host: any\n  inbound:\n    - port: any\n      proto: icmp\n      host: any\n    - port: any\n      proto: tcp\n      host: any \n";
  let _x = fs::write(&f2, &s).unwrap();
  config.clone().put_string("yaml", &s);
  
  let f2 = home.join("connection.json");
  let _x = fs::write(f2, &config.to_string()).unwrap();
}

config
}

