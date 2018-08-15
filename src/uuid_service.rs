extern crate uuid;

use uuid::Uuid;

pub fn make_id() -> String {
    uuid::Uuid::new_v4().to_string()
}