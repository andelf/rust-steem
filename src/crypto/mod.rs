use aes::Aes256;
use block_modes::block_padding::NoPadding;
use block_modes::{BlockMode, Cbc};
use sha2::{Digest, Sha256, Sha512};
use std::error::Error;
use std::mem;

pub use city::city_hash_crc_128;

mod city;

// create an alias for convinience
type Aes256Cbc = Cbc<Aes256, NoPadding>;

#[inline]
pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.input(input);
    unsafe { mem::transmute(hasher.result()) }
}
/*
#[inline]
pub fn keccak256(input: &[u8]) -> [u8: 32] {
    let mut hasher = Keccak256::new();
    hasher.input(input);
    unsafe {
        mem::transmute(hasher.result().into())
    }
}
*/

#[inline]
pub fn sha512(input: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.input(input);
    // NOTE: From<GenericArray<u8, 64>> is not impl-ed for [u8; 64]
    unsafe { mem::transmute(hasher.result()) }
}

pub fn aes_encrypt(key: &[u8], iv: &[u8], plain_text: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = plain_text.to_owned();

    // encrypt plaintext
    Aes256Cbc::new_var(key, iv)?.encrypt(&mut buffer, plain_text.len())?;

    Ok(buffer)
}

pub fn aes_decrypt(key: &[u8], iv: &[u8], cipher_text: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = cipher_text.to_owned();

    println!("dbeug iv {:?}", hex::encode(iv));

    Aes256Cbc::new_var(key, iv)?.decrypt(&mut buffer)?;

    Ok(buffer)
}
