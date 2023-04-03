use ndata::dataobject::*;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let ax = start_service(a0);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn start_service(servicename:String) -> String {
let mut da = DataArray::new();
da.push_string("sudo");
da.push_string("systemctl");
da.push_string("start");
da.push_string(&servicename);
let res = system_call(da);
let mut s = res.get_string("out") + & res.get_string("err");
if s == "".to_string() { s = "OK".to_string(); }

s
}

