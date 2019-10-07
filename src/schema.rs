table! {
    files (id) {
        id -> Integer,
        filepath -> Text,
        created -> Integer,
        updated -> Integer,
    }
}

table! {
    links (id) {
        id -> Integer,
        forward -> Text,
        created -> Integer,
        updated -> Integer,
    }
}

table! {
    texts (id) {
        id -> Integer,
        contents -> Text,
        created -> Integer,
        updated -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(files, links, texts,);
