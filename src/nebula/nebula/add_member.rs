use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use std::fs;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let a1 = o.get_string("peer");
let a2 = o.get_string("ipaddress");
let a3 = o.get_string("groups");
let ax = add_member(a0, a1, a2, a3);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn add_member(servicename:String, peer:String, ipaddress:String, groups:String) -> DataObject {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);
let ca_key = nbdir.join("ca.key");
let ca_crt = nbdir.join("ca.crt");
let _x = fs::copy(home.join("ca.key"), ca_key.to_owned());
let _x = fs::copy(home.join("ca.crt"), ca_crt.to_owned());

let bin = root.join("bin");
let mut da = DataArray::new();
da.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
da.push_string("sign");
da.push_string("-name");
da.push_string(&peer);
da.push_string("-ip");
da.push_string(&ipaddress);
if groups != "".to_string() {
  da.push_string("-group");
  da.push_string(&groups);
}
let res = system_call(da);
let _x = fs::remove_file(ca_key);
let _x = fs::remove_file(ca_crt);
let s = res.get_string("out") + & res.get_string("err");
if s != "".to_string() {
  let mut o = DataObject::new();
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

let ca_crt = home.join("ca.crt");
let home = home.join("members").join(&peer);
let _x = fs::create_dir_all(&home);
let host_crt = home.join("host.crt");
let host_key = home.join("host.key");
let _x = fs::rename(nbdir.join(peer.to_owned()+".crt"), &host_crt);
let _x = fs::rename(nbdir.join(peer.to_owned()+".key"), &host_key);

let mut jo = DataObject::new();
jo.put_string("ip_address", &ipaddress);
jo.put_string("groups", &groups);
let _x = fs::write(&home.join("info.json"), &jo.to_string()).unwrap();

jo.put_string("ca_crt", &fs::read_to_string(ca_crt).unwrap());
jo.put_string("host_crt", &fs::read_to_string(host_crt).unwrap());
jo.put_string("host_key", &fs::read_to_string(host_key).unwrap());

jo
}

