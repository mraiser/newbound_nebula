let mut o = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let bin = root.join("bin");
let mut sa = DataArray::new();
sa.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
sa.push_string("ca");
sa.push_string("-name");
sa.push_string(&name);
let res = system_call(sa);
let s = res.get_string("out") + & res.get_string("err");
if s != "".to_string() {
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

let ip_address = subnet.replace("X", "1");
let mut sa = DataArray::new();
sa.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
sa.push_string("sign");
sa.push_string("-name");
sa.push_string("host");
sa.push_string("-ip");
sa.push_string(&ip_address);
let res = system_call(sa);
let s = res.get_string("out") + & res.get_string("err");
if s != "".to_string() {
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

let home = root.join("networks").join(&name);
let _x = fs::create_dir_all(&home);
let _x = fs::rename(nbdir.join("ca.crt"), home.join("ca.crt"));
let _x = fs::rename(nbdir.join("ca.key"), home.join("ca.key"));
let _x = fs::rename(nbdir.join("host.crt"), home.join("host.crt"));
let _x = fs::rename(nbdir.join("host.key"), home.join("host.key"));

let mut jo = DataObject::new();
jo.put_object("lighthouses", DataObject::new());
jo.put_string("port", &port);
jo.put_string("subnet", &subnet);
jo.put_string("ip_address", &ip_address);
jo.put_string("host", "0.0.0.0");
jo.put_boolean("am_lighthouse", false);
jo.put_boolean("use_yaml", false);

save_config(name, jo)