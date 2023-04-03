use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;
use flowlang::flowlang::file::write_properties::write_properties;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("url");
let a1 = o.get_string("version");
let ax = install_release(a0, a1);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn install_release(url:String, version:String) -> String {
let nbdir = DataStore::new().root.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let build = root.join("bin");

println!("downloading {}", &url);
let resp = attohttpc::get(&url).send().unwrap();
if resp.is_success() {
  let response = resp.bytes().unwrap();
  let f = build.join("download.tgz");
  let _x = std::fs::create_dir_all(&build);
  let _x = std::fs::write(&f, &response).unwrap();
  println!("{:?}", f);
  let mut sa = DataArray::new();
  sa.push_string("tar");
  sa.push_string("-xzf");
  sa.push_string(&f.to_owned().into_os_string().into_string().unwrap());
  sa.push_string("-C");
  sa.push_string(&build.to_owned().into_os_string().into_string().unwrap());
  println!("{}", sa.to_string());
  let sa = system_call(sa);
  println!("{}", sa.to_string());
  let _x = std::fs::remove_file(&f);
  let pos = url.rfind('/').unwrap();
  let binary = &url[pos+1..];
  
  let mut p = DataObject::new();
  p.put_string("version", &version);
  p.put_string("binary", &binary);
  write_properties(build.join("version.txt").into_os_string().into_string().unwrap(), p);
  return "OK".to_string();
}

"ERROR: URL not found".to_string()
}

