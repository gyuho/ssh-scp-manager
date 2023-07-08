/// cargo run --example ssh_scp_manager
fn main() {
    // ref. https://github.com/env-logger-rs/env_logger/issues/47
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let (pk_encoded, pubkey_encoded) = ssh_scp_manager::rsa::new_key(Some(3072)).unwrap();
    println!("{pk_encoded}");
    println!("{pubkey_encoded}");
}
