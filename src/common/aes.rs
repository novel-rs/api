use std::{fs, path::Path};

use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit},
    Aes256,
};
use cbc::Decryptor;
use ring::{
    aead::{
        Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
        NONCE_LEN,
    },
    digest,
    error::Unspecified,
};

use crate::Error;

#[must_use]
struct CounterNonceSequence(u32);

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = vec![0; NONCE_LEN];

        let bytes = self.0.to_be_bytes();
        nonce_bytes[8..].copy_from_slice(&bytes);

        self.0 += 1;
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

pub(crate) fn aes_256_gcm_base64_encrypt<P, T, E>(
    mut data: String,
    file_path: P,
    password: T,
    aad: E,
) -> Result<(), Error>
where
    P: AsRef<Path>,
    T: AsRef<str>,
    E: AsRef<str>,
{
    let key = digest::digest(&digest::SHA256, password.as_ref().as_bytes());
    let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_ref())
        .map_err(|error| Error::Ring(error.to_string()))?;
    let associated_data = Aad::from(aad.as_ref().as_bytes());

    let mut sealing_key = SealingKey::new(unbound_key, CounterNonceSequence(1));

    sealing_key
        .seal_in_place_append_tag(associated_data, unsafe { data.as_mut_vec() })
        .map_err(|error| Error::Ring(error.to_string()))?;

    fs::write(file_path, base64_simd::STANDARD.encode_to_string(data))?;

    Ok(())
}

pub(crate) fn aes_256_gcm_base64_decrypt<P, T, E>(
    file_path: P,
    password: T,
    aad: E,
) -> Result<String, Error>
where
    P: AsRef<Path>,
    T: AsRef<str>,
    E: AsRef<str>,
{
    let mut data = fs::read(file_path)?;
    let data = base64_simd::STANDARD.decode_inplace(data.as_mut_slice())?;

    let key = digest::digest(&digest::SHA256, password.as_ref().as_bytes());
    let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_ref())
        .map_err(|error| Error::Ring(error.to_string()))?;
    let associated_data = Aad::from(aad.as_ref().as_bytes());

    let mut opening_key = OpeningKey::new(unbound_key, CounterNonceSequence(1));
    let decrypted_data = opening_key
        .open_in_place(associated_data, data)
        .map_err(|error| Error::Ring(error.to_string()))?;

    Ok(simdutf8::basic::from_utf8(decrypted_data)?.to_string())
}

pub(crate) fn aes_256_cbc_no_iv_base64_decrypt<T, E>(key: T, data: E) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
    E: AsRef<[u8]>,
{
    let decoded = base64_simd::STANDARD.decode_to_vec(data.as_ref())?;

    type Aes256CbcDec = Decryptor<Aes256>;
    let result = Aes256CbcDec::new(key.as_ref().into(), &[0; 16].into())
        .decrypt_padded_vec_mut::<Pkcs7>(&decoded)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_256_gcm_base64() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("aes.txt");

        let data = String::from("Hello World");
        aes_256_gcm_base64_encrypt(data, &file, "password", "aad")?;

        assert!(file.is_file());

        let decrypted = aes_256_gcm_base64_decrypt(file, "password", "aad")?;

        assert_eq!(decrypted, "Hello World");

        Ok(())
    }
}
