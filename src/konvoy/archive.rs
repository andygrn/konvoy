extern crate base64;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rust_sodium;

use std::io::{BufWriter, Cursor, Error as IoError, Read, Write};
use std::fs::File;
use std::path::Path;
use self::rust_sodium::crypto::sign;
use self::rust_sodium::crypto::sign::ed25519::{PublicKey, Signature};
use self::chrono::{DateTime, NaiveDateTime, Utc};
use self::regex::Regex;

#[derive(Debug)]
pub struct Archive {
    pub public_key: PublicKey,
    pub version: u8,
    pub datetime: DateTime<Utc>,
    pub signature: Signature,
    pub data: Vec<u8>,
}

impl Archive {
    pub fn from_name(name: &str) -> Result<Archive, ()> {
        lazy_static! {
            static ref NAME_REGEX: Regex = Regex::new(
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
        }
        let name_pieces = NAME_REGEX.captures(name);
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
            data: Vec::new(),
        })
    }

    pub fn from_stream(input_filename: &str, input_file: &mut Read) -> Result<Archive, ()> {
        let archive = Archive::from_name(&input_filename);
        if archive.is_err() {
            return Err(());
        }
        let mut archive = archive.unwrap();
        let read_result = input_file.read_to_end(&mut archive.data);
        if read_result.is_err() {
            return Err(());
        }
        Ok(archive)
    }

    pub fn write_to_disk(&self, path: &Path) -> Result<(), IoError> {
        let mut pathbuf = path.to_path_buf();
        pathbuf.push(self.get_filename() + &self.get_signature_base64());
        let mut file = BufWriter::new(File::create(pathbuf.as_path())?);
        file.write(&self.data)?;
        file.flush()?;
        Ok(())
    }

    pub fn get_filename(&self) -> String {
        format!(
            "{}{:03}{:010}",
            &self.get_public_key_base64(),
            &self.version,
            &self.datetime.timestamp()
        )
    }

    pub fn get_data_size(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn get_public_key_base64(&self) -> String {
        base64::encode_config(&self.public_key, base64::URL_SAFE_NO_PAD)
    }

    pub fn get_signature_base64(&self) -> String {
        base64::encode_config(&self.signature, base64::URL_SAFE_NO_PAD)
    }

    pub fn verify(&self) -> bool {
        let mut data_to_verify = Cursor::new(self.get_filename()).chain(Cursor::new(&self.data));
        let mut data_vec = Vec::new();
        data_to_verify.read_to_end(&mut data_vec).unwrap();
        sign::verify_detached(&self.signature, &data_vec, &self.public_key)
    }
}
