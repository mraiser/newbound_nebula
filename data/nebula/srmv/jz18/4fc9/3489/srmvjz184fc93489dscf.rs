let mut o = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let bin = root.join("bin");
let home = root.join("networks").join(&name);
let _x = fs::create_dir_all(&home);
let homestr = home.canonicalize().unwrap().into_os_string().into_string().unwrap();

// CA keypair, written straight into the network folder: the CA key never
// exists anywhere else.
let mut sa = DataArray::new();
sa.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
sa.push_string("ca");
sa.push_string("-name");
sa.push_string(&name);
sa.push_string("-out-crt");
sa.push_string(&(homestr.to_owned()+"/ca.crt"));
sa.push_string("-out-key");
sa.push_string(&(homestr.to_owned()+"/ca.key"));
let res = system_call(sa);
let s = res.get_string("out") + &res.get_string("err");
if s != "".to_string() {
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

// This host's own cert, signed by the new CA.
let ip_address = subnet.replace("X", "1");
let mut sa = DataArray::new();
sa.push_string(&bin.join("nebula-cert").into_os_string().into_string().unwrap());
sa.push_string("sign");
sa.push_string("-ca-crt");
sa.push_string(&(homestr.to_owned()+"/ca.crt"));
sa.push_string("-ca-key");
sa.push_string(&(homestr.to_owned()+"/ca.key"));
sa.push_string("-name");
sa.push_string("host");
sa.push_string("-ip");
sa.push_string(&ip_address);
sa.push_string("-out-crt");
sa.push_string(&(homestr.to_owned()+"/host.crt"));
sa.push_string("-out-key");
sa.push_string(&(homestr.to_owned()+"/host.key"));
let res = system_call(sa);
let s = res.get_string("out") + &res.get_string("err");
if s != "".to_string() {
  o.put_string("status", "err");
  o.put_string("msg", &s);
  return o;
}

let mut jo = DataObject::new();
jo.put_object("lighthouses", DataObject::new());
jo.put_string("port", &port);
jo.put_string("subnet", &subnet);
jo.put_string("ip_address", &ip_address);
jo.put_string("host", "0.0.0.0");
jo.put_boolean("am_lighthouse", false);
jo.put_boolean("use_yaml", false);

save_config(name, jo)
