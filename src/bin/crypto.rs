extern crate base64;
extern crate chrono;
extern crate rust_sodium;

use rust_sodium::crypto::sign;
use rust_sodium::crypto::sign::ed25519::{PublicKey, SecretKey, PUBLICKEYBYTES, SECRETKEYBYTES};
use std::io::{Read, Write};
use std::fs::OpenOptions;
use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};

fn main() {
    const FILENAME_VERSION: u8 = 1;
    let mut pk_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("id.public")
        .expect("Open public key file");

    let mut sk_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("id.secret")
        .expect("Open secret key file");

    let mut pk_string = String::with_capacity(PUBLICKEYBYTES);
    let mut sk_string = String::with_capacity(SECRETKEYBYTES);
    pk_file
        .read_to_string(&mut pk_string)
        .expect("Read public key file");
    sk_file
        .read_to_string(&mut sk_string)
        .expect("Read secret key file");

    if pk_string.len() == 0 && sk_string.len() == 0 {
        println!("No ID found, generating a new one...");
        let (pk, sk) = sign::gen_keypair();
        pk_string = encode_config(&pk[..], URL_SAFE_NO_PAD);
        sk_string = encode_config(&sk[..], URL_SAFE_NO_PAD);
        pk_file
            .write_all(pk_string.as_bytes())
            .expect("Write public key to file");
        sk_file
            .write_all(sk_string.as_bytes())
            .expect("Write secret key to file");
        println!("Your new public ID is {}", pk_string);
    }

    let pk = decode_config(&pk_string, URL_SAFE_NO_PAD).expect("Decode public key file");
    let sk = decode_config(&sk_string, URL_SAFE_NO_PAD).expect("Decode secret key file");

    let pk = PublicKey::from_slice(pk.as_slice()).expect("Decode public key");
    let sk = SecretKey::from_slice(sk.as_slice()).expect("Decode secret key");

    let now: DateTime<Utc> = Utc::now();
    let archive_meta = format!(
        "{}{:03}{:010}",
        &pk_string,
        &FILENAME_VERSION,
        &now.timestamp()
    );
    let data_to_verify = archive_meta + &String::from("some data");

    println!("{:?}", data_to_verify);
    let signature = sign::sign_detached(&data_to_verify.as_bytes(), &sk);
    let signature_string = encode_config(&signature[..], URL_SAFE_NO_PAD);
    println!("{:?}", signature_string);
    let data_is_valid = sign::verify_detached(&signature, &data_to_verify.as_bytes(), &pk);
    assert!(data_is_valid);
}
