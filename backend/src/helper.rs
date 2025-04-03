use crc32fast::Hasher as Crc32Hasher;
use ic_agent::export::Principal;
use sha2::{Digest, Sha224};

// principal_to_account_id
pub fn principal_to_account_id(principal: &Principal, subaccount: Option<[u8; 32]>) -> [u8; 32] {
    let subaccount = subaccount.unwrap_or([0u8; 32]);

    // Concatenate the domain separator, the principal bytes, and the subaccount.
    let mut hasher = Sha224::new();
    hasher.update(b"\x0Aaccount-id"); // Domain separator
    hasher.update(principal.as_slice());
    hasher.update(subaccount);
    let hash = hasher.finalize(); // This gives 28 bytes.

    // Compute the CRC32 checksum of the hash.
    let mut crc_hasher = Crc32Hasher::new();
    crc_hasher.update(hash.as_slice());
    let checksum = crc_hasher.finalize();

    // Prepend the 4-byte checksum to the 28-byte hash.
    let mut account_id = [0u8; 32];
    account_id[..4].copy_from_slice(&checksum.to_be_bytes());
    account_id[4..].copy_from_slice(hash.as_slice());

    account_id
}

// is_valid_account_id
pub fn is_valid_account_id(account_id_hex: &str) -> Result<bool, Box<dyn std::error::Error>> {
    if account_id_hex.len() != 64 {
        return Ok(false);
    }

    let account_bytes = hex::decode(account_id_hex)?;
    if account_bytes.len() != 32 {
        return Ok(false);
    }

    let mut hasher = Crc32Hasher::new();
    hasher.update(&account_bytes[4..]); // bytes[4..32]
    let computed_checksum = hasher.finalize();

    let provided_checksum =
        u32::from_be_bytes([account_bytes[0], account_bytes[1], account_bytes[2], account_bytes[3]]);

    Ok(computed_checksum == provided_checksum)
}
