//! Encoding of this token format:
//! ```txt
//! token = "T1." + base64(data + hmac_sha256(data))
//! data = seconds_since_epoch:u64 user:access+
//! access = id:u32 app_id:u16 _:u8 can_write
//! can_write = 0x0000 | 0x0001
//! ```
//! All fields are encoded as big endian unsigned integers.
//! The base 64 encoding ignore padding, and use URL alphabet.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::prelude::*;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::{hmac::Hmac, sha2::Sha256};

use crate::Result;
use crate::errs::TOKEN_BASE64;
use crate::errs::TOKEN_OBSOLETE;
use crate::errs::TOKEN_PREFIX;
use crate::errs::TOKEN_SIGNATURE;
use crate::errs::TOKEN_WRONG_LENGTH;
use crate::op::Token;
use crate::op::TokenItem;

/// Maximal age of a token: 7 day.
/// The value can change.
const MAX_AGE: u64 = 7 * 24 * 60 * 60;

pub fn token_encode(mut token_str: String, token: &Token, key: &[u8], now: u64) -> String {
    let mut buff = [0; 8 + 8 + 64 * Token::ACCESS_LEN + 32];
    buff[0..8].copy_from_slice(&now.to_be_bytes());
    buff[8..12].copy_from_slice(&token.uid.to_be_bytes());
    buff[12..16].copy_from_slice(&u32::from(token.is_admin).to_be_bytes());

    let mut w = 16;
    for item in token.access.iter().take_while(|item| item.id != 0) {
        buff[w + 0..w + 4].copy_from_slice(&item.id.to_be_bytes());
        buff[w + 4..w + 6].copy_from_slice(&item.app.to_be_bytes());
        buff[w + 6..w + 8].copy_from_slice(&u16::from(item.can_write).to_be_bytes());
        w += 8;
    }

    let mut h = Hmac::new(Sha256::new(), key);
    h.input(&buff[..w]);
    buff[w..w + 32].copy_from_slice(h.result().code());

    token_str.push_str("T1.");
    URL_SAFE_NO_PAD.encode_string(&buff[..w + 32], &mut token_str);

    token_str
}

pub fn token_decode(key: &[u8], now: u64, s: &str) -> Result<Token> {
    let b64_data = s.strip_prefix("T1.").ok_or(TOKEN_PREFIX)?;

    let mut raw_data = [0; 8 + 8 + 8 * Token::ACCESS_LEN + 32];
    let raw_data_len = URL_SAFE_NO_PAD
        .decode_slice(b64_data, &mut raw_data)
        .map_err(|_| TOKEN_BASE64)?;

    // Check signature
    let bytes_data = &raw_data[..raw_data_len - 32];
    let signature = &raw_data[raw_data_len - 32..][..32];
    let mut h = Hmac::new(Sha256::new(), key);
    h.input(bytes_data);
    if h.result() != MacResult::new(signature) {
        Err(TOKEN_SIGNATURE)?;
    }

    // Transfert as a slice of u64
    let u64_data = {
        if bytes_data.len() % 8 != 0 {
            TOKEN_WRONG_LENGTH.push(format!("size: {} is not multiple of 8", bytes_data.len()))?;
        } else if bytes_data.len() < 16 {
            TOKEN_WRONG_LENGTH.push(format!("size: {} too short (>8)", bytes_data.len()))?;
        } else if bytes_data.len() / 8 > Token::ACCESS_LEN + 2 {
            TOKEN_WRONG_LENGTH.push(format!(
                "size: {} too long 8*(2+{})",
                bytes_data.len(),
                Token::ACCESS_LEN
            ))?;
        }
        let mut u64_data = [0u64; 2 + Token::ACCESS_LEN];
        for i in 0..(bytes_data.len() / 8) {
            let mut buff = [0u8; 8];
            buff.copy_from_slice(&bytes_data[i * 8..][..8]);
            u64_data[i] = u64::from_be_bytes(buff);
        }
        u64_data
    };

    // Check date
    if u64_data[0] + MAX_AGE < now {
        Err(TOKEN_OBSOLETE)?;
    }

    // Decode
    let uid = (u64_data[1] >> 32) as u32;
    let is_admin = u64_data[1] & 0xFF != 0;
    let mut access = [TokenItem::default(); Token::ACCESS_LEN];
    for (i, &item) in u64_data[2..].iter().enumerate() {
        access[i].id = (item >> 32) as u32;
        access[i].app = ((item >> 16) & 0xFFFF) as u16;
        access[i].can_write = item & 0xFF != 0;
    }

    Ok(Token {
        uid,
        is_admin,
        access,
    })
}

#[test]
fn test_token_encoding() {
    let token_str =
        "T1.AAAAAGon4CcAAAABAAAAATMzRERWeAAAO7OyxgbJnCtUhnLlvikp_jWABGZcVjlmVypWSqn4gd8";
    let key = b"key";
    let now = 1780998183;

    let token = Token {
        uid: 1,
        is_admin: true,
        access: {
            let mut access = [TokenItem::default(); Token::ACCESS_LEN];
            access[0] = TokenItem {
                id: 0x3333_4444,
                app: 0x5678,
                can_write: false,
            };
            access
        },
    };

    assert_eq!(token_str, token_encode(String::new(), &token, key, now));
    assert_eq!(token_decode(key, now, token_str).unwrap(), token);
}
