use hex_simd::AsciiCase;
use md5::{Digest, Md5};
use ring::digest;

pub fn md5_hex<T>(data: T, ascii_case: AsciiCase) -> String
where
    T: AsRef<[u8]>,
{
    let mut hasher = Md5::new();
    hasher.update(data.as_ref());

    hex_simd::encode_to_string(hasher.finalize(), ascii_case)
}

pub fn sha256<T>(data: T) -> digest::Digest
where
    T: AsRef<[u8]>,
{
    digest::digest(&digest::SHA256, data.as_ref())
}
