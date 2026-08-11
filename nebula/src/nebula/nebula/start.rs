use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::io::Read;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["servicename"] {
        if !o.has(p) {
            let mut e = DataObject::new();
            e.put_string("status", "err");
            e.put_string("msg", &format!("missing required parameter: {}", p));
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", e);
            return result_obj;
        }
    }
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let arg_0: String = o.get_string("servicename");
        start(arg_0)
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_object("a", ax);
            result_obj
        }
        Err(err) => {
            let mut err_obj = DataObject::new();
            err_obj.put_string("status", "err");

            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };

            err_obj.put_string("msg", &msg);
            // Wrapped in the same `a` envelope a successful return uses.
            // Unwrapped, callers that unpack the envelope (newbound's
            // format_result, for one) report an opaque 500 — "Not an object:
            // DString(\"err\")" — instead of this message.
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", err_obj);
            result_obj
        }
    }
}

pub fn start(servicename: String) -> DataObject {
let mut g = flowlang::datastore::DataStore::globals();
if !g.has("nebulaservices") { g.put_object("nebulaservices", DataObject::new()); }
let mut g = g.get_object("nebulaservices");
if !g.has(&servicename) { 
  let mut s = DataObject::new();
  s.put_boolean("running", false);
  g.put_object(&servicename, s); 
}
let mut g = g.get_object(&servicename);

if g.get_boolean("running"){
  println!("nebula network {} already running", &servicename);
}
else {
  let store = DataStore::new().root;
  let nbdir = store.parent().unwrap().to_owned();
  let root = nbdir.join("runtime").join("nebula");
  let root = root.canonicalize().unwrap();
  let bin = root.join("bin").join("nebula");
  let config = root.join("networks").join(&servicename).join("config.yml");
  let config = config.display().to_string();

  let mut args = Vec::new();
  args.push("-config");
  args.push(&config);
  let cmd = Command::new(&bin.display().to_string())
    .args(args)
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn();

  let mut cmd = cmd.unwrap();
  let id = cmd.id();
  println!("STARTING {} as process #{}", &servicename, id);

  g.put_boolean("running", true);

  {
    let mut g = g.clone();
    let mut stdout = cmd.stdout.take().unwrap();
    let servicename = servicename.clone();
    thread::spawn(move || {
      let mut buf: [u8; 1024] = [0; 1024];

      while g.get_boolean("running") {
        let n = stdout.read(&mut buf).unwrap();
        if n == 0 {
          break;
        }
        else {
          let s = std::str::from_utf8(&buf[0..n]).unwrap();
          print!("{}",s);
        }
      }
      println!("nebula network {} OUT DONE", &servicename);
      g.put_boolean("running", false);
    });
  }
  {
    let mut g = g.clone();
    let mut stderr = cmd.stderr.take().unwrap();
    let servicename = servicename.clone();
    thread::spawn(move || {
      let mut buf: [u8; 1024] = [0; 1024];

      while g.get_boolean("running") {
        let n = stderr.read(&mut buf).unwrap();
        if n == 0 {
          break;
        }
        else {
          let s = std::str::from_utf8(&buf[0..n]).unwrap();
          print!("{}",s);
        }
      }
      println!("nebula network {} ERR DONE", &servicename);
      g.put_boolean("running", false);
    });
  }
}

DataObject::new()
}
