let mut jo = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");
let f = build.join("version.txt");

// systemd is optional: containers, macOS and plain-process installs run
// tunnels supervised instead. Report which world this host is in so the
// UI can hide the systemd controls where they can only fail.
let has_systemd = Path::new("/run/systemd/system").exists();
jo.put_boolean("systemd", has_systemd);

// networks named in the start= key of botd.properties auto-start at boot
let mut bootlist: Vec<String> = Vec::new();
let bp = root.join("botd.properties");
if bp.exists() {
  let p = read_properties(bp.into_os_string().into_string().unwrap());
  if p.has("start") {
    for s in p.get_string("start").split(",") {
      let s = s.trim();
      if s != "" { bootlist.push(s.to_string()); }
    }
  }
}

// supervised tunnel state lives in this process's globals (start/stop)
let g = DataStore::globals();
let sup = match g.has("nebulaservices") { true => Some(g.get_object("nebulaservices")), false => None };

if f.exists() {
  let p = read_properties(f.into_os_string().into_string().unwrap());
  jo.put_string("tag_name", &p.get_string("version"));
  jo.put_string("binary_name", &p.get_string("binary"));

  let home = root.join("networks");
  let _x = fs::create_dir_all(&home);
  let mut networks = DataArray::new();
  jo.put_array("networks", networks.clone());
  for file in fs::read_dir(&home).unwrap() {
    let f2 = file.unwrap();
    let name = f2.file_name().into_string().unwrap();
    let f2 = f2.path();
    if f2.join("ca.crt").exists() {
      let mut jo2 = DataObject::new();
      networks.push_object(jo2.clone());
      jo2.put_string("name", &name);

      let mut service = false;
      let mut service_running = false;
      if has_systemd {
        service = Path::new(&("/etc/systemd/system/".to_string()+&name+".service")).exists();
        if service {
          let mut da = DataArray::new();
          da.push_string("systemctl");
          da.push_string("is-active");
          da.push_string(&name);
          service_running = system_call(da).get_string("out").trim() == "active".to_string();
        }
      }
      jo2.put_boolean("service", service);
      jo2.put_boolean("service_running", service_running);

      let mut supervised = false;
      if let Some(s) = &sup {
        if s.has(&name) { supervised = s.get_object(&name).get_boolean("running"); }
      }
      jo2.put_boolean("supervised", supervised);
      jo2.put_boolean("running", service_running || supervised);
      jo2.put_boolean("boot", bootlist.contains(&name));

      let owner;
      if f2.join("ca.key").exists() { owner = "local".to_string(); }
      else { owner = fs::read_to_string(f2.join("owner.txt")).unwrap().trim().to_owned(); }
      jo2.put_string("owner", &owner);
      let jo3 = DataObject::from_string(&fs::read_to_string(f2.join("connection.json")).unwrap());
      jo2.put_object("config", jo3);
    }
  }
}
else {
  jo.put_string("tag_name", "Not Installed");
  jo.put_string("binary_name", "N/A");
}

jo