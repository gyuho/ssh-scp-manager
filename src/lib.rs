use std::{
    env,
    io::{self, Error, ErrorKind},
};

use lazy_static::lazy_static;
use primitive_types::{H160, H256, U256};
use rand::Rng;
use ring::rand::{SecureRandom, SystemRandom};

/// Generates a random string of length "n".
pub fn secure_string(n: usize) -> String {
    let bytes = secure_bytes(n).unwrap();
    let mut d = bs58::encode(&bytes[..]).into_string();
    if n > 0 && d.len() > n {
        d.truncate(n);
    }
    d
}

/// Generates a random string of length "n".
pub fn secure_bytes(n: usize) -> io::Result<Vec<u8>> {
    let mut d: Vec<u8> = vec![0u8; n];
    secure_random()
        .fill(&mut d)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random.fill {}", e)))?;
    Ok(d)
}

fn secure_random() -> &'static dyn SecureRandom {
    use std::ops::Deref;
    lazy_static! {
        static ref RANDOM: SystemRandom = SystemRandom::new();
    }
    RANDOM.deref()
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_secure_string --exact --show-output
#[test]
fn test_secure_string() {
    use log::info;
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let word1 = secure_string(100);
    let word2 = secure_string(100);

    assert_eq!(word1.len(), 100);
    assert_eq!(word2.len(), 100);
    assert_ne!(word1, word2);

    info!("word1: {:?}", word1);
    info!("word2: {:?}", word2);
}

/// Returns a file path randomly generated in tmp directory.
/// The file does not exist yet.
pub fn tmp_path(n: usize, sfx: Option<&str>) -> io::Result<String> {
    let tmp_dir = env::temp_dir();
    let tmp_file_path = tmp_dir.join(format!("{}{}", secure_string(n), sfx.unwrap_or("")));
    let tmp_file_path = tmp_file_path.as_os_str().to_str().unwrap();
    Ok(String::from(tmp_file_path))
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_temp_path --exact --show-output
#[test]
fn test_temp_path() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let p1 = tmp_path(10, Some(".zstd")).unwrap();
    let p2 = tmp_path(10, Some(".zstd")).unwrap();
    assert_ne!(p1, p2);

    log::info!("p1: {:?}", p1);
    log::info!("p2: {:?}", p2);
}

pub fn secure_u8() -> io::Result<u8> {
    let mut d: Vec<u8> = vec![0u8; 1];
    secure_random()
        .fill(&mut d)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random.fill {}", e)))?;
    Ok(d[0])
}

pub fn usize() -> usize {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn u8() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn u16() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn u32() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn u64() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_uints --exact --show-output
#[test]
fn test_uints() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let v1 = usize();
    let v2 = usize();
    assert_ne!(v1, v2);
    log::info!("v1: {v1}");
    log::info!("v2: {v2}");

    let v1 = u8();
    let v2 = u8();
    assert_ne!(v1, v2);
    log::info!("v1: {v1}");
    log::info!("v2: {v2}");

    let v1 = u16();
    let v2 = u16();
    assert_ne!(v1, v2);
    log::info!("v1: {v1}");
    log::info!("v2: {v2}");

    let v1 = u32();
    let v2 = u32();
    assert_ne!(v1, v2);
    log::info!("v1: {v1}");
    log::info!("v2: {v2}");

    let v1 = u64();
    let v2 = u64();
    assert_ne!(v1, v2);
    log::info!("v1: {v1}");
    log::info!("v2: {v2}");
}

pub fn secure_h160() -> io::Result<H160> {
    // MUST BE slice.len() < 4 * 5
    let mut d: Vec<u8> = vec![0u8; 20];
    secure_random()
        .fill(&mut d)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random.fill {}", e)))?;
    Ok(H160::from_slice(&d))
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_secure_h160 --exact --show-output
#[test]
fn test_secure_h160() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let v1 = secure_h160().unwrap();
    let v2 = secure_h160().unwrap();
    assert_ne!(v1, v2);

    log::info!("v1: 0x{:x}", v1);
    log::info!("v2: 0x{:x}", v2);
}

pub fn secure_h256() -> io::Result<H256> {
    let mut d: Vec<u8> = vec![0u8; 32];
    secure_random()
        .fill(&mut d)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random.fill {}", e)))?;
    Ok(H256::from_slice(&d))
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_secure_h256 --exact --show-output
#[test]
fn test_secure_h256() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let v1 = secure_h256().unwrap();
    let v2 = secure_h256().unwrap();
    assert_ne!(v1, v2);

    log::info!("v1: 0x{:x}", v1);
    log::info!("v2: 0x{:x}", v2);
}

pub fn secure_u256() -> io::Result<U256> {
    // MUST BE slice.len() < 4 * 8
    let mut d: Vec<u8> = vec![0u8; 32];
    secure_random()
        .fill(&mut d)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secure_random.fill {}", e)))?;
    Ok(U256::from_big_endian(&d))
}

/// RUST_LOG=debug cargo test --package random-manager --lib -- test_secure_u256 --exact --show-output
#[test]
fn test_secure_u256() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let v1 = secure_u256().unwrap();
    let v2 = secure_u256().unwrap();
    assert_ne!(v1, v2);

    log::info!("v1: {v1}");
    log::info!("v2: {v2}");
}
