//admin
#[derive(Debug, PartialEq, Queryable)]
pub struct Admin {
    pub aid: i32,
    pub name: String,
    pub password: String,
}