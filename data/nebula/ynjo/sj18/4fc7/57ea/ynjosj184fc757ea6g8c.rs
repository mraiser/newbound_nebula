let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let f = root.join("networks").join(&servicename);
let f2 = f.join("connection.json");
let jo = DataObject::from_string(&fs::read_to_string(f2).unwrap());

let mut shm = "".to_string();
let mut hosts = "".to_string();
let lhs = jo.get_object("lighthouses");
for peer in lhs.clone().keys(){
  let lh = lhs.get_object(&peer);
  shm = shm+"  \""+(&lh.get_string("private_ip"))+"\": [\""+(&lh.get_string("public_ip"))+":"+(&lh.get_string("port"))+"\"]\n";
  hosts = hosts+"    - \""+(&lh.get_string("private_ip"))+"\"\n";
}

let path = &f.canonicalize().unwrap().into_os_string().into_string().unwrap();
let s = "pki:\n  ca: ".to_string()+path+"/ca.crt\n  cert: "+path+"/host.crt\n  key: "+path+"/host.key\nstatic_host_map:\n"+(&shm)+"lighthouse:\n  am_lighthouse: "+(&jo.get_boolean("am_lighthouse").to_string())+"\n  interval: 60\n  hosts:\n"+(&hosts)+"listen:\n  host: "+(&jo.get_string("host"))+"\n  port: "+(&jo.get_int("port").to_string())+"\npunchy: true\ntun:\n  dev: "+(&servicename)+"\n  drop_local_broadcast: false\n  drop_multicast: false\n  tx_queue: 500\n  mtu: 1300\n  routes:\n  unsafe_routes:\nlogging:\n  level: info\n  format: text\nfirewall:\n  conntrack:\n    tcp_timeout: 120h\n    udp_timeout: 3m\n    default_timeout: 10m\n    max_connections: 100000\n  outbound:\n    - port: any\n      proto: any\n      host: any\n  inbound:\n    - port: any\n      proto: icmp\n      host: any\n    - port: any\n      proto: tcp\n      host: any \n";

s