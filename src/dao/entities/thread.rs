//Thread
use crate::dao::schema::thread;

//type threads = Vec<Thread>;

#[derive(Debug, PartialEq, Queryable, Copy, Clone)]
//#[primary_key(tid)]
pub struct Thread {
    pub tid: i32,
    pub first_floor: i32,
}

#[derive(Insertable)]
#[table_name="thread"]
pub struct NewThread {
    pub first_floor: i32,
}
