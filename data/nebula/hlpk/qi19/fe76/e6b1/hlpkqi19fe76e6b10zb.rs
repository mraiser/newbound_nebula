let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let home = nbdir.join("runtime").join("nebula").join("networks").join(&servicename).join("members").join(&peer);
if !home.exists() { return "ERROR: No such member".to_string(); }
let _x = std::fs::remove_dir_all(&home);

"OK".to_string()
