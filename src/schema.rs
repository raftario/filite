table! {
    files (id) {
        id -> Integer,
        filepath -> Text,
        created -> Integer,
    }
}

table! {
    links (id) {
        id -> Integer,
        forward -> Text,
        created -> Integer,
    }
}

table! {
    texts (id) {
        id -> Integer,
        contents -> Text,
        created -> Integer,
        highlight -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(files, links, texts,);
