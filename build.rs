use rand::Rng;
use std::{env, fs::File, io::Write, path::Path};

fn main() {
    let mut key = [0; 32];
    let mut rng = rand::thread_rng();
    for b in key.iter_mut() {
        *b = rng.gen();
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("key");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(&key).unwrap();
}
