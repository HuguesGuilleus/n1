use std::io::Write;

use base64::prelude::*;
use crypto::mac::Mac;
use crypto::util::fixed_time_eq;
use crypto::{hmac::Hmac, sha2::Sha256};

use crate::op::{Token, TokenLevel};
use crate::{Result, errs};

/// Maximal age of a token.
const MAX_AGE: u64 = 7 * 24 * 60 * 60;

/**
 * Encoding of this token format:
 * ```txt
 * token = "T0." + base64(data + hmac-sha256(data))
 * data = seconds_since_Epoch:u64 user:access groups:access*
 * access= id:u32 level:u8
 * ```
 */
pub fn token_encode(token: &Token, key: &[u8], now: u64) -> String {
    let mut hasher = Hmac::new(Sha256::new(), key);
    let mut data: Vec<u8> =
        Vec::with_capacity(8 + 5 + 5 * 5 + 5 * token.groups_vec.len() + hasher.output_bytes());

    data.write(&now.to_be_bytes()).unwrap();

    data.write(&token.uid.to_be_bytes()).unwrap();
    data.push(token.global as u8);

    for &(id, level) in token.groups_array.iter().chain(token.groups_vec.iter()) {
        data.write(&id.to_be_bytes()).unwrap();
        data.push(level as u8);
    }
    let data_size = data.len();

    hasher.input(&data);
    data.resize(data_size + hasher.output_bytes(), 0);
    hasher.raw_result(&mut data[data_size..]);

    let mut s = String::from("T0.");
    BASE64_URL_SAFE_NO_PAD.encode_string(data, &mut s);

    s
}

// Decode a token. See [encode] for format detail.
pub fn token_decode(s: &str, key: &[u8], now: u64) -> Result<Token> {
    const SHA256_BYTES: usize = 32;

    let s = s.strip_prefix("T0.").ok_or(errs::TOKEN_PREFIX)?;
    let data = BASE64_URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| errs::TOKEN_BASE64)?;

    // check size
    if data.len() < 8 + 5 + SHA256_BYTES {
        return Err(errs::TOKEN_WRONG_LENGTH);
    }

    // Check signature
    let mut hasher = Hmac::new(Sha256::new(), key);
    hasher.input(&data[0..data.len() - SHA256_BYTES]);
    let mut signature = [0; SHA256_BYTES];
    hasher.raw_result(&mut signature);
    if !fixed_time_eq(&signature, &data[data.len() - SHA256_BYTES..]) {
        return Err(errs::TOKEN_SIGNATURE);
    }
    let mut data = &data[0..data.len() - SHA256_BYTES];

    // Check timee
    let mut buf_u64 = [0u8; 8];
    std::io::Read::read_exact(&mut data, &mut buf_u64).unwrap();
    let time = u64::from_be_bytes(buf_u64);
    if time < now && now + MAX_AGE < time {
        return Err(errs::TOKEN_OBSOLETE);
    }

    // TOKEN_WRONG_LENGTH
    let chuncks = data.chunks_exact(5);
    if chuncks.remainder().len() != 0 {
        return Err(errs::TOKEN_WRONG_LENGTH);
    }
    let mut parts = chuncks.map(|chunck| {
        let mut buf_u32 = [0u8; 4];
        buf_u32.copy_from_slice(&chunck[0..4]);

        TokenLevel::try_from(chunck[4]).map(|level| (u32::from_be_bytes(buf_u32), level))
    });

    let (uid, global) = match parts.next() {
        None => return Err(errs::TOKEN_WRONG_LENGTH),
        Some(Err(err)) => return Err(err),
        Some(Ok((uid, global))) => (uid, global),
    };

    let groups_array = [
        parts.next().unwrap_or(Ok((0, TokenLevel::None)))?,
        parts.next().unwrap_or(Ok((0, TokenLevel::None)))?,
        parts.next().unwrap_or(Ok((0, TokenLevel::None)))?,
        parts.next().unwrap_or(Ok((0, TokenLevel::None)))?,
        parts.next().unwrap_or(Ok((0, TokenLevel::None)))?,
    ];
    let groups_vec = parts.collect::<Result<Vec<_>>>()?;

    Ok(Token {
        uid,
        global,
        groups_array,
        groups_vec,
    })
}

impl TryFrom<u8> for TokenLevel {
    type Error = errs::Error;
    fn try_from(value: u8) -> errs::Result<Self> {
        match value {
            1 => Ok(TokenLevel::Read),
            2 => Ok(TokenLevel::Write),
            3 => Ok(TokenLevel::Admin),
            _ => Err(errs::TOKEN_LEVEL),
        }
    }
}

#[test]
fn test_token_level_order() {
    assert!(TokenLevel::None < TokenLevel::Read);
    assert!(TokenLevel::None < TokenLevel::Write);
    assert!(TokenLevel::None < TokenLevel::Admin);

    assert!(TokenLevel::Read < TokenLevel::Write);
    assert!(TokenLevel::Read < TokenLevel::Admin);
    assert!(TokenLevel::Write < TokenLevel::Admin);
}

#[test]
fn test_token_encode() {
    let str_token = "T0.AAAAAGon4CcSNFZ4Aqq7zAEBqrvMAgKqu8wDAqq7zAQDqrvMBQOqu8wGA-AW4ZwuphTMM0RgKWqkOJGVSSeeKXIzNHI6P5Abdbpg";

    let token = Token {
        uid: 0x12_34_56_78,
        global: TokenLevel::Write,
        groups_array: [
            (0xAA_BB_CC_01, TokenLevel::Read),
            (0xAA_BB_CC_02, TokenLevel::Write),
            (0xAA_BB_CC_03, TokenLevel::Write),
            (0xAA_BB_CC_04, TokenLevel::Admin),
            (0xAA_BB_CC_05, TokenLevel::Admin),
        ],
        groups_vec: vec![(0xAA_BB_CC_06, TokenLevel::Admin)],
    };

    assert_eq!(str_token, token_encode(&token, b"key", 1780998183));
    assert_eq!(Ok(token), token_decode(str_token, b"key", 1780998183 + 10));
}
