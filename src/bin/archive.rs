extern crate base64;
extern crate chrono;
extern crate regex;
extern crate rust_sodium;

use self::rust_sodium::crypto::sign;
use self::rust_sodium::crypto::sign::ed25519::{PublicKey, Signature};
use self::chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug)]
pub struct Archive {
    pub public_key: PublicKey,
    pub version: u8,
    pub datetime: DateTime<Utc>,
    pub signature: Signature,
    pub data: String,
}

impl Archive {
    pub fn from_str(input_str: &str) -> Result<Archive, ()> {
        let name_regex = regex::Regex::new(
            r"(?x)
            ^(
                [A-Za-z0-9_\-]{43} # Public key base64
            )(
                \d{3}              # Archive version
            )(
                \d{10}             # Unix timestamp
            )(
                [A-Za-z0-9_\-]{86} # Archive signature base64
            )$
        ",
        ).unwrap();

        let name_pieces = name_regex.captures(&input_str[..142]);
        if name_pieces.is_none() {
            return Err(());
        }

        let name_pieces = name_pieces.unwrap();
        let archive_public_key = name_pieces.get(1);
        let archive_version = name_pieces.get(2);
        let archive_datetime = name_pieces.get(3);
        let archive_signature = name_pieces.get(4);
        if archive_public_key.is_none() || archive_version.is_none() || archive_datetime.is_none()
            || archive_signature.is_none()
        {
            return Err(());
        }

        let public_key = base64::decode_config(
            &archive_public_key.unwrap().as_str(),
            base64::URL_SAFE_NO_PAD,
        );
        if public_key.is_err() {
            return Err(());
        }
        let public_key = PublicKey::from_slice(public_key.unwrap().as_slice());
        if public_key.is_none() {
            return Err(());
        }

        let version = u8::from_str_radix(archive_version.unwrap().as_str(), 10);
        if version.is_err() {
            return Err(());
        }

        // let datetime = archive_datetime.unwrap();
        let datetime = i64::from_str_radix(archive_datetime.unwrap().as_str(), 10);
        if datetime.is_err() {
            return Err(());
        }
        let datetime =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(datetime.unwrap(), 0), Utc);

        let signature = base64::decode_config(
            &archive_signature.unwrap().as_str(),
            base64::URL_SAFE_NO_PAD,
        );
        if signature.is_err() {
            return Err(());
        }
        let signature = Signature::from_slice(signature.unwrap().as_slice());
        if signature.is_none() {
            return Err(());
        }

        Ok(Archive {
            public_key: public_key.unwrap(),
            version: version.unwrap(),
            datetime: datetime,
            signature: signature.unwrap(),
            // data: String::from(&input_str[142..]),
            data: String::from("some data"),
        })
    }

    pub fn verify(&self) -> bool {
        let public_key_base64 = base64::encode_config(&self.public_key, base64::URL_SAFE_NO_PAD);
        let archive_meta = format!(
            "{}{:03}{:010}",
            &public_key_base64,
            &self.version,
            &self.datetime.timestamp()
        );
        let data_to_verify = archive_meta + &String::from("some data");
        sign::verify_detached(
            &self.signature,
            &data_to_verify.as_bytes(),
            &self.public_key,
        )
    }
}
