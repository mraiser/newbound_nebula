use ndata::dataobject::*;
use flowlang::datastore::DataStore;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let ax = members(a0);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn members(servicename:String) -> DataObject {
let mut o = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);

let f = home.join("members");
if f.exists(){
  for file in std::fs::read_dir(&f).unwrap() {
    let path = file.unwrap().path();
    let name = &path.display().to_string();
    let f2 = path.join("info.json");
    if f2.exists(){
      let s = std::fs::read_to_string(f2).unwrap();
      let jo2 = DataObject::from_string(&s);
      o.put_object(&name, jo2);
    }
  }
}

o
}

