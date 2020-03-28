table! {
    admin (aid) {
        aid -> Int4,
        name -> Bpchar,
        password -> Bpchar,
    }
}

table! {
    post (pid) {
        pid -> Int4,
        auth -> Nullable<Bpchar>,
        pdate -> Timestamptz,
        subject -> Nullable<Bpchar>,
        comment -> Nullable<Text>,
        sid -> Int4,
        tid -> Int4,
    }
}

table! {
    subsection (sid) {
        sid -> Int4,
        name -> Bpchar,
    }
}

table! {
    thread (tid) {
        tid -> Int4,
        first_floor -> Int4,
    }
}

joinable!(post -> thread (tid));

allow_tables_to_appear_in_same_query!(
    admin,
    post,
    subsection,
    thread,
);
