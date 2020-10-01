use crate::config::Config;
use sled::Db;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::http::StatusCode;

fn setup() -> (&'static Config, &'static Db) {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
        .ok();

    let config = Box::leak(Box::new(Default::default()));
    let db = Box::leak(Box::new(
        sled::Config::default().temporary(true).open().unwrap(),
    ));

    crate::db::insert_user("user", "password", true, db, config).unwrap();

    (config, db)
}

const AUTH: &str = "Basic dXNlcjpwYXNzd29yZA==";

#[tokio::test(core_threads = 2)]
async fn file() {
    let (config, db) = setup();
    let filter = crate::routes::handler(config, db);

    let value = b"file";

    let reply = warp::test::request()
        .path("/f")
        .method("POST")
        .body(value)
        .header("Authorization", AUTH)
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::CREATED);

    let id = std::str::from_utf8(reply.body()).unwrap();
    let reply = warp::test::request()
        .path(&format!("/{}", id))
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::OK);
    assert_eq!(
        reply
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap(),
        "application/octet-stream"
    );
    assert_eq!(reply.body().as_ref(), value);
}

#[tokio::test(core_threads = 2)]
async fn link() {
    let (config, db) = setup();
    let filter = crate::routes::handler(config, db);

    let value = "https://google.com/";

    let reply = warp::test::request()
        .path("/l")
        .method("POST")
        .body(value)
        .header("Authorization", AUTH)
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::CREATED);

    let id = std::str::from_utf8(reply.body()).unwrap();
    let reply = warp::test::request()
        .path(&format!("/{}", id))
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        reply.headers().get("Location").unwrap().to_str().unwrap(),
        value
    );
}

#[tokio::test(core_threads = 2)]
async fn text() {
    let (config, db) = setup();
    let filter = crate::routes::handler(config, db);

    let value = "text";

    let reply = warp::test::request()
        .path("/t")
        .method("POST")
        .body(value)
        .header("Authorization", AUTH)
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::CREATED);

    let id = std::str::from_utf8(reply.body()).unwrap();
    let reply = warp::test::request()
        .path(&format!("/{}", id))
        .reply(&filter)
        .await;
    assert_eq!(reply.status(), StatusCode::OK);
    assert_eq!(std::str::from_utf8(reply.body()).unwrap(), value);
}
