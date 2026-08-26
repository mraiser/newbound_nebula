use ndata::dataobject::DataObject;
use flowlang::datastore::DataStore;
use std::process::{Command, Stdio, Child};
use std::thread;
use std::io::Read;
use std::time::Duration;
use std::sync::{LazyLock, Mutex};
use std::collections::HashMap;

// The supervised tunnel processes, keyed by network name. Shared with stop
// (crate::nebula::nebula::start::NEBULA_CHILDREN).
pub static NEBULA_CHILDREN: LazyLock<Mutex<HashMap<String, Child>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

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
let store = DataStore::new().root;
let nbdir = store.parent().unwrap().to_owned();
let root = nbdir.join("runtime").join("nebula");
let bin = root.join("bin").join("nebula");
let confpath = root.join("networks").join(&servicename).join("config.yml");

let mut o = DataObject::new();

if !bin.exists() {
  o.put_string("status", "err");
  o.put_string("msg", "The nebula engine is not installed on this instance - install a release first.");
  return o;
}
if !confpath.exists() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("No configuration for network {} - save its configuration first.", servicename));
  return o;
}

let mut g = DataStore::globals();
if !g.has("nebulaservices") { g.put_object("nebulaservices", DataObject::new()); }
let mut g = g.get_object("nebulaservices");
if !g.has(&servicename) {
  let mut s = DataObject::new();
  s.put_boolean("running", false);
  g.put_object(&servicename, s);
}
let mut g = g.get_object(&servicename);

// Reap a finished child so a stale entry can't block a restart.
{
  let mut kids = NEBULA_CHILDREN.lock().unwrap();
  if let Some(child) = kids.get_mut(&servicename) {
    if let Ok(Some(_)) = child.try_wait() {
      kids.remove(&servicename);
      g.put_boolean("running", false);
    }
  }
}

if g.get_boolean("running") {
  o.put_string("status", "ok");
  o.put_boolean("already", true);
  o.put_string("msg", &format!("{} is already running", servicename));
  return o;
}

let config = confpath.canonicalize().unwrap().display().to_string();
let cmd = Command::new(&bin.canonicalize().unwrap().display().to_string())
  .args(["-config", &config])
  .stderr(Stdio::piped())
  .stdout(Stdio::piped())
  .spawn();
if cmd.is_err() {
  o.put_string("status", "err");
  o.put_string("msg", &format!("Could not launch the nebula binary: {}", cmd.err().unwrap()));
  return o;
}
let mut cmd = cmd.unwrap();
let id = cmd.id();
println!("STARTING {} as process #{}", &servicename, id);
g.put_boolean("running", true);
g.put_int("pid", id as i64);
g.put_string("log", "");

// Both output streams echo to the Newbound log AND keep a bounded tail in
// the globals, so failures have something honest to show.
fn tail_reader(mut stream: impl Read, mut g: DataObject, servicename: String, last: bool) {
  let mut buf: [u8; 1024] = [0; 1024];
  loop {
    let n = match stream.read(&mut buf) { Ok(n) => n, Err(_) => 0 };
    if n == 0 { break; }
    let s = String::from_utf8_lossy(&buf[0..n]).to_string();
    print!("{}", s);
    let mut cur = g.get_string("log");
    cur.push_str(&s);
    if cur.len() > 4000 {
      let mut cut = cur.len() - 4000;
      while !cur.is_char_boundary(cut) { cut += 1; }
      cur = cur[cut..].to_string();
    }
    g.put_string("log", &cur);
  }
  if last {
    println!("nebula network {} exited", &servicename);
    g.put_boolean("running", false);
  }
}
{
  let g = g.clone();
  let stdout = cmd.stdout.take().unwrap();
  let sn = servicename.clone();
  thread::spawn(move || { tail_reader(stdout, g, sn, true); });
}
{
  let g = g.clone();
  let stderr = cmd.stderr.take().unwrap();
  let sn = servicename.clone();
  thread::spawn(move || { tail_reader(stderr, g, sn, false); });
}

NEBULA_CHILDREN.lock().unwrap().insert(servicename.to_owned(), cmd);

// Grace check: nebula dies within milliseconds on a bad config or a missing
// tun device - catch that and report it instead of claiming success.
thread::sleep(Duration::from_millis(700));
let mut died = false;
{
  let mut kids = NEBULA_CHILDREN.lock().unwrap();
  if let Some(child) = kids.get_mut(&servicename) {
    if let Ok(Some(_)) = child.try_wait() {
      died = true;
      kids.remove(&servicename);
    }
  }
}
if died {
  g.put_boolean("running", false);
  let log = g.get_string("log");
  o.put_string("status", "err");
  o.put_string("msg", &format!("nebula exited immediately: {}", log.trim()));
  return o;
}

o.put_string("status", "ok");
o.put_boolean("already", false);
o.put_string("msg", &format!("{} is running (pid {})", servicename, id));
o
}
