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