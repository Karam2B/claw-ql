use claw_ql::{macros::Collection, operations::dynamic_client::DynamicClient};

#[derive(Collection)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

fn main() {
    let client = DynamicClient::default().add_collection::<Todo>();


}
