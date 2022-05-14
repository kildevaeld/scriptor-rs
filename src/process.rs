use std::collections::HashMap;

use rquickjs::IntoJs;

#[derive(IntoJs)]
pub struct Process {
    argv: Vec<String>,
    env: HashMap<String, String>,
}
