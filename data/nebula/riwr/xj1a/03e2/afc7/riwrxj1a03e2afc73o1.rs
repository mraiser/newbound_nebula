let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let _x = fs::create_dir_all(&root);
let bp = root.join("botd.properties");

// read the whole file, preserving every key we don't own
let mut lines: Vec<String> = Vec::new();
let mut start: Vec<String> = Vec::new();
let mut seen = false;
if bp.exists() {
  for line in fs::read_to_string(&bp).unwrap().lines() {
    if line.trim_start().starts_with("start=") {
      seen = true;
      for s in line.trim_start()[6..].split(",") {
        let s = s.trim();
        if s != "" { start.push(s.to_string()); }
      }
      lines.push("start=".to_string()); // placeholder, rewritten below
    } else {
      lines.push(line.to_string());
    }
  }
}
if !seen { lines.push("start=".to_string()); }

start.retain(|s| s != &servicename);
if enabled { start.push(servicename.to_owned()); }
let startline = format!("start={}", start.join(","));
let text = lines.iter().map(|l| if l == "start=" { startline.clone() } else { l.clone() })
  .collect::<Vec<String>>().join("\n") + "\n";
fs::write(&bp, &text).unwrap();

let mut o = DataObject::new();
o.put_string("status", "ok");
o.put_string("start", &start.join(","));
o.put_string("msg", if enabled { "will start at boot" } else { "will not start at boot" });
o