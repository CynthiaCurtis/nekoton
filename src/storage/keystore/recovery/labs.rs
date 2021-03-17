use anyhow::Error;
use bip39::{Language, Seed};
use tiny_hderive::bip32::ExtendedPrivKey;

use super::GeneratedKey;

pub fn derive_from_words(phrase: &str, account_id: u16) -> Result<ed25519_dalek::Keypair, Error> {
    let mnemonic = bip39::Mnemonic::from_phrase(phrase, Language::English)?;
    let hd = Seed::new(&mnemonic, "");
    let seed_bytes = hd.as_bytes();

    let derived = ExtendedPrivKey::derive(
        seed_bytes,
        format!("m/44'/396'/0'/0/{}", account_id).as_str(),
    )
    .map_err(|e| Error::msg(format!("{:#?}", e)))?;

    ed25519_keys_from_secret_bytes(&derived.secret()) //todo check me
}

pub fn generate_words(entropy: [u8; 32], account_id: u16) -> Result<GeneratedKey, Error> {
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy, Language::English)?
        .phrase()
        .to_string();
    Ok(GeneratedKey {
        keypair: derive_from_words(&mnemonic, account_id)?,
        words: mnemonic.split_whitespace().map(|x| x.to_string()).collect(),
    })
}

fn ed25519_keys_from_secret_bytes(bytes: &[u8]) -> Result<ed25519_dalek::Keypair, Error> {
    let secret = ed25519_dalek::SecretKey::from_bytes(bytes).map_err(|e| {
        Error::msg(format!(
            "failed to import ton secret key. {}",
            e.to_string()
        ))
    })?;

    let public = ed25519_dalek::PublicKey::from(&secret);

    Ok(ed25519_dalek::Keypair { secret, public })
}

#[cfg(test)]
mod test {
    use crate::recovery::ton_labs::derive_from_words_labs;

    #[test]
    fn bad_mnemonic() {
        let key = derive_from_words_labs(
            "pioneer fever hazard scam install wise reform corn bubble leisure amazing note",
        );
        assert!(key.is_err());
    }

    #[test]
    fn ton_recovery() {
        let key = derive_from_words_labs(
            "pioneer fever hazard scan install wise reform corn bubble leisure amazing note",
        )
        .unwrap();
        let secret = key.secret;

        let target_secret = ed25519_dalek::SecretKey::from_bytes(
            &hex::decode("e371ef1d7266fc47b30d49dc886861598f09e2e6294d7f0520fe9aa460114e51")
                .unwrap(),
        )
        .unwrap();

        assert_eq!(secret.as_bytes(), target_secret.as_bytes())
    }
}
