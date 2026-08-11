let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let f = root.join("networks").join(&servicename).join("connection.json");
let jo = DataObject::from_string(&fs::read_to_string(f).unwrap());

// one template: save_config renders (and re-persists) the YAML
let jo = save_config(servicename, jo);

jo.get_string("yaml")
