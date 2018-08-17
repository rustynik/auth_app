extern crate uuid;

use uuid::Uuid;

pub struct UuidMaker;

impl MakeId for UuidMaker {
    pub fn make_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}