use des::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit},
    Des,
};
use ecb::{Decryptor, Encryptor};

use crate::Error;

pub(crate) fn des_ecb_base64_encrypt<T, E>(key: T, data: E) -> Result<String, Error>
where
    T: AsRef<[u8]>,
    E: AsRef<[u8]>,
{
    type DesEcbEnc = Encryptor<Des>;
    let result = DesEcbEnc::new(key.as_ref().into()).encrypt_padded_vec_mut::<Pkcs7>(data.as_ref());

    Ok(base64_simd::STANDARD.encode_to_string(result))
}

pub(crate) fn des_ecb_base64_decrypt<T, E>(key: T, data: E) -> Result<String, Error>
where
    T: AsRef<[u8]>,
    E: AsRef<[u8]>,
{
    let data = base64_simd::STANDARD.decode_to_vec(data)?;

    type DesEcbDec = Decryptor<Des>;
    let result = DesEcbDec::new(key.as_ref().into()).decrypt_padded_vec_mut::<Pkcs7>(&data)?;

    Ok(simdutf8::basic::from_utf8(&result)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn des() -> Result<(), Error> {
        let key = "abcdefgh";
        let data = "Hello World";

        let encrypted = des_ecb_base64_encrypt(key, data)?;
        let decrypted = des_ecb_base64_decrypt(key, encrypted)?;

        assert_eq!(decrypted, data);

        Ok(())
    }
}
