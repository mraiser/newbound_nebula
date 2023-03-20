let mut da = DataArray::new();
da.push_string("service");
da.push_string(&servicename);
da.push_string("restart");
let res = system_call(da);
let mut s = res.get_string("out") + & res.get_string("err");
if s == "".to_string() { s = "OK".to_string(); }

s