use byteorder::{ByteOrder, LE};
use secp256k1::ffi;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;
use std::env;
use crypto::{aes_decrypt, city_hash_crc_128, sha256, sha512};

pub mod crypto;
#[allow(unused_imports, dead_code)]
pub mod message;

// const REMOTE_ADDR: &str = "node.mahdiyari.info:2001";
// const REMOTE_ADDR: &str = "192.168.1.162:2001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr = env::args().skip(1).next().expect("usage: cargo run -- ip:addr");

    let secp = Secp256k1::new();
    let mut stream = TcpStream::connect(&remote_addr).await?;
    let mut rng = rand::thread_rng();

    // let my_priv_key = secp256k1::key::ONE_KEY; // :) no need to tweak
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
    println!("! tweak pub key  => {:?}", remote_pub_key);

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
    println!("! packet payload size = {}", size);
    println!("! packet message type = {}", msg_type);

    let payload = &decrypted[8..];

    let mut cursor = 0;

    println!(
        "user_agent: {:?}",
        String::from_utf8_lossy(&payload[1..1 + payload[cursor] as usize])
    );
    cursor += 1 + payload[cursor] as usize;

    println!(
        "core_protocol_version: {}",
        LE::read_u32(&payload[cursor..cursor + 4]),
    );
    cursor += 4;

    println!(
        "inbound_address: {:?}",
        payload[cursor..cursor + 4].iter().rev().collect::<Vec<_>>()
    );
    cursor += 4;

    println!(
        "inbound_port: {}",
        LE::read_u16(&payload[cursor..cursor + 2]),
    );
    cursor += 2;

    println!(
        "outbound_port: {}",
        LE::read_u16(&payload[cursor..cursor + 2]),
    );
    cursor += 2;

    println!(
        "node_public_key: {:?}",
        hex::encode(&payload[cursor..cursor + 33])
    );
    cursor += 33;

    println!(
        "signed_shared_secret: {:?}",
        hex::encode(&payload[cursor..cursor + 65])
    );
    cursor += 65;

    let user_data_raw = &payload[cursor..];

    let user_data = message::parse_variant(user_data_raw);
    println!(
        "user_data =>\n{}",
        serde_json::to_string_pretty(&user_data)?
    );

    Ok(())
}
