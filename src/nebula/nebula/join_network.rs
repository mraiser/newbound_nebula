use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use crate::nebula::nebula::save_config::save_config;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let a1 = o.get_string("subnet");
let a2 = o.get_string("ipaddress");
let a3 = o.get_string("port");
let a4 = o.get_string("owner");
let a5 = o.get_string("ca_crt");
let a6 = o.get_string("host_crt");
let a7 = o.get_string("host_key");
let a8 = o.get_object("lighthouses");
let a9 = o.get_string("groups");
let ax = join_network(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn join_network(servicename:String, subnet:String, ipaddress:String, port:String, owner:String, ca_crt:String, host_crt:String, host_key:String, lighthouses:DataObject, groups:String) -> DataObject {
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

