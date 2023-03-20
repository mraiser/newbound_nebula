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