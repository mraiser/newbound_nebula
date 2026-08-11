let store = DataStore::new().root;
let nbdir = store.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let assets = store.join("nebula").join("_ASSETS");
let unit = std::fs::read_to_string(assets.join("service.txt")).unwrap();
let unit = unit.replace("SERVICENAME", &servicename);
let unit = unit.replace("ROOTDIR", &root.canonicalize().unwrap().into_os_string().into_string().unwrap());

let tmp = root.join("networks").join(&servicename).join(&(servicename.to_owned()+".service"));
std::fs::write(&tmp, &unit).unwrap();

let mut s = "".to_string();

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("mv");
sa.push_string(&tmp.into_os_string().into_string().unwrap());
sa.push_string(&("/etc/systemd/system/".to_string()+&servicename+".service"));
let res = system_call(sa);
s = s + &res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("daemon-reload");
let res = system_call(sa);
s = s + &res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("enable");
sa.push_string(&servicename);
let res = system_call(sa);
s = s + &res.get_string("out") + &res.get_string("err");

if s == "" { s = "OK".to_string(); }

s
