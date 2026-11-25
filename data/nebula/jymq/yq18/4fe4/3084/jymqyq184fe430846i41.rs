let mut o = DataObject::new();

let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let home = root.join("networks").join(&servicename);

let users = crate::API.security.security.users();

let f = home.join("members");
if f.exists(){
  for file in std::fs::read_dir(&f).unwrap() {
    let path = file.unwrap().path();
    let name = path.file_name().unwrap().to_os_string().to_str().unwrap().to_owned();
    let f2 = path.join("info.json");
    if f2.exists(){
      let s = std::fs::read_to_string(f2).unwrap();
      let mut jo2 = DataObject::from_string(&s);
      
      if users.has(&name) {
        let u = users.get_object(&name);
        let n = u.get_string("displayname");
        jo2.put_string("name", &n);
      }
      
      // expiry + fingerprint for the list (Phase 3); phones record them at
      // signing, peers added earlier get them from their cert
      if (!jo2.has("not_after") || !jo2.has("fingerprint")) && path.join("host.crt").exists() {
        if let Ok(out) = std::process::Command::new(root.join("bin").join("nebula-cert"))
            .args(["print", "-json", "-path", &path.join("host.crt").display().to_string()]).output() {
          let s = String::from_utf8_lossy(&out.stdout).to_string();
          for key in ["notAfter", "fingerprint"] {
            let pat = format!("\"{}\":\"", key);
            if let Some(i) = s.find(&pat) {
              let rest = &s[i + pat.len()..];
              if let Some(j) = rest.find('"') { jo2.put_string(if key == "notAfter" { "not_after" } else { "fingerprint" }, &rest[..j]); }
            }
          }
        }
      }
      o.put_object(&name, jo2);
    }
  }
}

o
