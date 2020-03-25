use byteorder::{ByteOrder, LE};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;
// secp256k1_ec_pubkey_tweak_mul;
use secp256k1::ffi;
// use secp256k1::key::ONE_KEY;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use crypto::{aes_decrypt, city_hash_crc_128, sha256, sha512};

pub mod crypto;

const REMOTE_ADDR: &str = "node.mahdiyari.info:2001";
// const REMOTE_ADDR: &str = "192.168.1.162:2001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let secp = Secp256k1::new();
    let mut stream = TcpStream::connect(REMOTE_ADDR).await?;
    let mut rng = rand::thread_rng();

    //let my_priv_key = ONE_KEY;
    let my_priv_key = SecretKey::new(&mut rng);
    let my_pub_key = PublicKey::from_secret_key(&secp, &my_priv_key);

    // println!("priv {:?}", my_priv_key);

    // Step 1: send my public key to remote
    let raw_message = my_pub_key.serialize();

    println!("! => {}", hex::encode(&raw_message[..]));
    stream.write_all(&raw_message[..]).await?;

    // Step 2: recv remote public key
    let mut buffer = [0u8; 4096];
    let n = stream.read(&mut buffer[..]).await?;

    let mut remote_pub_key = {
        assert_eq!(n, 33, "compressed public key is 33 bytes");
        let buf = &buffer[..n];
        println!("! <= {}", hex::encode(buf));
        PublicKey::from_slice(buf)?
    };
    println!("! remote pub key => {:?}", remote_pub_key);

    // Step 3: get shared secret, generate AES parameters
    // tweak pub key
    assert!(
        unsafe {
            ffi::secp256k1_ec_pubkey_tweak_mul(
                *secp.ctx(),
                remote_pub_key.as_mut_ptr(),
                my_priv_key.as_ptr(),
            )
        } != 0
    );
    println!("! tweek pub key  => {:?}", remote_pub_key);

    // 64 bytes
    let shared_secret = sha512(&remote_pub_key.serialize()[1..]);

    println!("! shared_key => {:?}", hex::encode(&shared_secret[..]));

    let aes_key = sha256(&shared_secret[..]);
    // fc::city_hash_crc_128, modified version of google's cityhash
    let iv = city_hash_crc_128(&shared_secret[..]);

    println!("! aes key => {:?}", hex::encode(&aes_key));
    println!("! aes iv  => {:?}", hex::encode(&iv));

    // Rocks! Now read packet
    let n = stream.read(&mut buffer[..]).await?;
    let buf = &buffer[..n];
    // println!("! <= {}", hex::encode(buf));

    let decrypted = aes_decrypt(&aes_key, &iv, buf)?;
    println!("! decrypted <= {}", hex::encode(&decrypted));
    // println!("! decrypted <= {}", String::from_utf8_lossy(&decrypted));

    println!("! header <= {}", hex::encode(&decrypted[..8]));
    let size = LE::read_u32(&decrypted[..4]);
    let msg_type = LE::read_u32(&decrypted[4..8]);
    println!("packet payload size = {}", size);
    println!("packet message type = {}", msg_type);

    // let payload = &decrypted[8..];

    Ok(())
}