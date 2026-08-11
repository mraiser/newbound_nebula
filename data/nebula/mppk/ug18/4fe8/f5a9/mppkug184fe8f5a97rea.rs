let mut da = DataArray::new();
da.push_string("sudo");
da.push_string("systemctl");
da.push_string("start");
da.push_string(&servicename);
let res = system_call(da);
let mut s = res.get_string("out") + & res.get_string("err");
if s == "".to_string() { s = "OK".to_string(); }

s