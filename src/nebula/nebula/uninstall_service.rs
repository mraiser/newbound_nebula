use ndata::dataobject::*;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let ax = uninstall_service(a0);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn uninstall_service(servicename:String) -> String {
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

