use std::env;

#[tokio::main]
async fn main() {
    if env::var_os("FILITE_LOG").is_none() {
        env::set_var("FILITE_LOG", "INFO");
    }
    env_logger::init_from_env("FILITE_LOG");
}
