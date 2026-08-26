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
  let myip = config.get_string("ip_address");
  let myip = myip.split("/").next().unwrap().to_string();

  let mut shm = "".to_string();
  let mut hosts = "".to_string();
  let lhs = config.get_object("lighthouses");
  for peer in lhs.clone().keys() {
    let lh = lhs.get_object(&peer);
    let pip = lh.get_string("private_ip");
    let pip = pip.split("/").next().unwrap();
    if pip == myip { continue; }
    shm = shm + "  \""+pip+"\": [\""+(&lh.get_string("public_ip"))+":"+(&lh.get_string("port"))+"\"]\n";
    hosts = hosts + "    - \""+pip+"\"\n";
  }

  // The peer-fed mesh: every member's candidate endpoints, learned over the
  // Newbound peer network (see endpoints/update_hosts). With punchy on both
  // sides, members hole-punch each other directly - no public lighthouse
  // needed. A host never lists itself, and lighthouse entries win.
  if config.has("hosts") {
    let hm = config.get_object("hosts");
    for ip in hm.clone().keys() {
      let plain = ip.split("/").next().unwrap().to_string();
      if plain == myip || lhs.has(&ip) { continue; }
      let mut eps: Vec<String> = Vec::new();
      for e in hm.get_array(&ip).objects() {
        let e = e.string();
        if e != "" { eps.push(format!("\"{}\"", e)); }
      }
      if eps.len() == 0 { continue; }
      shm = shm + "  \""+(&plain)+"\": ["+(&eps.join(", "))+"]\n";
    }
  }

  // The one home of the config.yml template - build_config renders through here.
  let path = &home.canonicalize().unwrap().into_os_string().into_string().unwrap();
  let s = "pki:\n  ca: ".to_string()+path+"/ca.crt\n  cert: "+path+"/host.crt\n  key: "+path+"/host.key\nstatic_host_map:\n"+(&shm)+"lighthouse:\n  am_lighthouse: "+(&config.get_boolean("am_lighthouse").to_string())+"\n  interval: 60\n  hosts:\n"+(&hosts)+"listen:\n  host: "+(&config.get_string("host"))+"\n  port: "+(&config.get_string("port"))+"\npunchy:\n  punch: true\n  respond: true\ntun:\n  dev: "+(&servicename)+"\n  drop_local_broadcast: false\n  drop_multicast: false\n  tx_queue: 500\n  mtu: 1300\n  routes:\n  unsafe_routes:\nlogging:\n  level: info\n  format: text\nfirewall:\n  conntrack:\n    tcp_timeout: 120h\n    udp_timeout: 3m\n    default_timeout: 10m\n    max_connections: 100000\n  outbound:\n    - port: any\n      proto: any\n      host: any\n  inbound:\n    - port: any\n      proto: icmp\n      host: any\n    - port: any\n      proto: tcp\n      host: any\n";
  let _x = fs::write(&f2, &s).unwrap();
  config.put_string("yaml", &s);

  let f2 = home.join("connection.json");
  let _x = fs::write(f2, &config.to_string()).unwrap();
}

config