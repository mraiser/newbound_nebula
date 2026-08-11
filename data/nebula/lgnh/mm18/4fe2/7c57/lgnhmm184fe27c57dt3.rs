let store = DataStore::new().root;
let nbdir = store.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let assets = store.join("nebula").join("_ASSETS");
let s = std::fs::read_to_string(assets.join("service.txt")).unwrap();
let s = s.replace("SERVICENAME", &servicename);
let s = s.replace("ROOTDIR", &root.canonicalize().unwrap().into_os_string().into_string().unwrap());
let cmd = root.join("networks").join(&servicename).join(&(servicename.to_owned()+".service")).into_os_string().into_string().unwrap(); // "/etc/systemd/system/".to_string()+&servicename+".service";
let f = Path::new(&cmd);
std::fs::write(f, &s).unwrap();

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("mv");
sa.push_string(&cmd);
sa.push_string(&("/etc/systemd/system/".to_string()+&servicename+".service"));
let res = system_call(sa);
let s = s + &res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("daemon-reload");
let res = system_call(sa);
let s = s + &res.get_string("out") + &res.get_string("err");

let mut sa = DataArray::new();
sa.push_string("sudo");
sa.push_string("systemctl");
sa.push_string("enable");
sa.push_string(&servicename);
let res = system_call(sa);
let mut s = s + &res.get_string("out") + & res.get_string("err");

if s == "" { s = "OK".to_string(); }

s