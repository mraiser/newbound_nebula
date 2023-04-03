use ndata::dataobject::*;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("servicename");
let ax = restart_service(a0);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn restart_service(servicename:String) -> String {
let mut da = DataArray::new();
da.push_string("service");
da.push_string(&servicename);
da.push_string("restart");
let res = system_call(da);
let mut s = res.get_string("out") + & res.get_string("err");
if s == "".to_string() { s = "OK".to_string(); }

s
}

