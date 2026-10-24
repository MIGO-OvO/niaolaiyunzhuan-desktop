use base64::{engine::general_purpose::STANDARD, Engine};
use minisign_verify::{PublicKey, Signature};
use serde_json::Value;
use std::{error::Error, fs, path::PathBuf};

fn verify(
    bytes: &[u8],
    public_key: &PublicKey,
    signature: &Signature,
    version: &str,
) -> Result<(), Box<dyn Error>> {
    public_key.verify(bytes, signature, true)?;
    let signed_version = signature
        .trusted_comment()
        .split('\t')
        .find_map(|part| part.strip_prefix("version:"))
        .ok_or("Signature does not bind the release version")?;
    if signed_version != version {
        return Err("Release version must match the signed version".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Pass the release directory")?,
    );
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json"))?;
    let manifest: Value = serde_json::from_slice(&fs::read(directory.join("latest.json"))?)?;
    let version = manifest["version"].as_str().ok_or("Missing version")?;
    assert_eq!(Some(version), config["version"].as_str());
    let platform = &manifest["platforms"]["windows-x86_64"];
    let installer = format!("niaolaiyunzhuan_{version}_x64-setup.exe");
    assert_eq!(
        platform["url"],
        format!("https://github.com/MIGO-OvO/niaolaiyunzhuan-desktop/releases/download/v{version}/{installer}")
    );
    let encoded_signature = platform["signature"].as_str().ok_or("Missing signature")?;
    assert_eq!(
        encoded_signature,
        fs::read_to_string(directory.join(format!("{installer}.sig")))?.trim()
    );
    let public_key = PublicKey::decode(&String::from_utf8(
        STANDARD.decode(
            config["plugins"]["updater"]["pubkey"]
                .as_str()
                .ok_or("Missing public key")?,
        )?,
    )?)?;
    let signature = Signature::decode(&String::from_utf8(STANDARD.decode(encoded_signature)?)?)?;
    let mut bytes = fs::read(directory.join(installer))?;
    verify(&bytes, &public_key, &signature, version)?;
    assert!(
        verify(&bytes, &public_key, &signature, "9999.0.0").is_err(),
        "Inflated manifest version must fail verification"
    );
    bytes[0] ^= 1;
    assert!(
        verify(&bytes, &public_key, &signature, version).is_err(),
        "Tampered installer must fail verification"
    );
    println!("v{version}: signature and signed version verified; tampered installer rejected");
    Ok(())
}
