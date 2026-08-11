let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");
let _x = std::fs::create_dir_all(&build);

let f = build.join("download.tgz");
let fstr = f.to_owned().into_os_string().into_string().unwrap();

// no http-client dependency: curl over system_call, like the platform's
// own git/zip/unzip/tar calls
let mut sa = DataArray::new();
sa.push_string("curl");
sa.push_string("-fsSL");
sa.push_string("-o");
sa.push_string(&fstr);
sa.push_string(&url);
let res = system_call(sa);
let err = res.get_string("err");
if !f.exists() || std::fs::metadata(&f).unwrap().len() == 0 {
  let _x = std::fs::remove_file(&f);
  return "ERROR: Download failed: ".to_string()+&err;
}

let mut sa = DataArray::new();
sa.push_string("tar");
sa.push_string("-xzf");
sa.push_string(&fstr);
sa.push_string("-C");
sa.push_string(&build.to_owned().into_os_string().into_string().unwrap());
let res = system_call(sa);
let err = res.get_string("err");
let _x = std::fs::remove_file(&f);
if err != "".to_string() { return "ERROR: ".to_string()+&err; }

let pos = url.rfind('/').unwrap();
let binary = &url[pos+1..];

let mut p = DataObject::new();
p.put_string("version", &version);
p.put_string("binary", &binary);
write_properties(build.join("version.txt").into_os_string().into_string().unwrap(), p);

"OK".to_string()
