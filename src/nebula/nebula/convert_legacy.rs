use ndata::dataobject::*;

pub fn execute(_o: DataObject) -> DataObject {
let ax = convert_legacy();
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn convert_legacy() -> String {
"NO".to_string()
}

