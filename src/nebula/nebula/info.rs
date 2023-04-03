use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use flowlang::flowlang::file::read_properties::read_properties;
use std::fs;
use ndata::dataarray::DataArray;
use std::path::Path;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(_o: DataObject) -> DataObject {
let ax = info();
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn info() -> DataObject {
let mut jo = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");
let f = build.join("version.txt");
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
      jo2.put_boolean("service", Path::new(&("/etc/systemd/system/".to_string()+&name+".service")).exists());
      
      let mut da = DataArray::new();
      da.push_string("systemctl");
      da.push_string("is-active");
      da.push_string(&name);
      let b = system_call(da).get_string("out").trim() == "active".to_string();
      jo2.put_boolean("running", b);
      
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
}

