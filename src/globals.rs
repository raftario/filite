pub const KEY: &[u8; 32] = include_bytes!(concat!(env!("OUT_DIR"), "/key"));

lazy_static! {
    pub static ref EMPTY_HASH: Vec<u8> = crate::setup::hash(b"");
    pub static ref POOL: crate::Pool =
        crate::setup::create_pool(&CONFIG.database_url, CONFIG.pool_size);
}

#[cfg(feature = "dev")]
lazy_static! {
    pub static ref CONFIG: crate::setup::Config = crate::setup::Config::debug();
    pub static ref PASSWORD_HASH: Vec<u8> = {
        dotenv::dotenv().ok();
        let password = crate::get_env!("PASSWD");
        crate::setup::hash(password.as_bytes())
    };
}

#[cfg(not(feature = "dev"))]
lazy_static! {
    pub static ref CONFIG: crate::setup::Config =
        crate::setup::init(std::env::args().fold(0, |mode, a| if &a == "init" {
            2
        } else if &a == "passwd" {
            1
        } else {
            mode
        }));
    pub static ref PASSWORD_HASH: Vec<u8> = {
        let password_path = crate::setup::get_password_path();
        std::fs::read(&password_path).unwrap_or_else(|e| {
            eprintln!("Can't read password hash from disk: {}", e);
            std::process::exit(1);
        })
    };
}
