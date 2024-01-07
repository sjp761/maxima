use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{bail, Result};

use base64::{engine::general_purpose, Engine};
use openssl::symm::{decrypt, encrypt, Cipher};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::core::endpoints::API_PROXY_NOVAFUSION_LICENSES;

pub const OOA_CRYPTO_KEY: [u8; 16] = [
    65, 50, 114, 45, 208, 130, 239, 176, 220, 100, 87, 197, 118, 104, 202, 9,
];

const LICENSE_PATH: &str = "ProgramData/Electronic Arts/EA Services/License";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct License {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub signature: String,
    pub cipher_key: String,
    pub machine_hash: String,
    pub content_id: String,
    pub user_id: String,
    pub game_token: Option<String>,
    pub grant_time: String,
    pub start_time: String,
}

pub async fn request_license(
    content_id: &str,
    machine_hash: &str,
    access_token: &str,
    request_token: Option<&str>,
    request_type: Option<&str>,
) -> Result<License> {
    let mut query = Vec::new();
    query.push(("contentId", content_id));
    query.push(("machineHash", machine_hash));
    query.push(("ea_eadmtoken", access_token));

    if request_token.is_some() {
        query.push(("requestToken", request_token.unwrap()));
        query.push(("requestType", request_type.unwrap()));
    }

    let res = Client::new()
        .get(API_PROXY_NOVAFUSION_LICENSES)
        .query(&query)
        .header("X-Requester-Id", "Origin Online Activation")
        .header("User-Agent", "EACTransaction")
        .send()
        .await?;
    if res.status() != StatusCode::OK {
        bail!("License request failed");
    }

    let signature = res.headers().get("x-signature").unwrap().to_owned();
    let body: Vec<u8> = res.bytes().await?.to_vec();

    let mut license = decrypt_license(body.as_slice())?;
    license.signature = signature.to_str()?.to_owned();

    Ok(license)
}

pub fn decrypt_license(data: &[u8]) -> Result<License> {
    let cipher = Cipher::aes_128_cbc();
    let decrypted_data = decrypt(cipher, &OOA_CRYPTO_KEY, Some(&[0; 16]), data)?;
    let data = String::from_utf8(decrypted_data)?;
    Ok(quick_xml::de::from_str(data.as_str())?)
}

pub fn save_license(license: &License, path: PathBuf) -> Result<()> {
    let mut data = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>".to_string();
    data.push_str(quick_xml::se::to_string(license)?.as_str());

    if !data.contains("<GameToken>") {
        data.remove_matches("<GameToken/>");
    }

    let cipher = Cipher::aes_128_cbc();
    let encrypted_data = encrypt(cipher, &OOA_CRYPTO_KEY, Some(&[0; 16]), data.as_bytes())?;

    let decode_b64 = false;

    let mut signature = license.signature.as_bytes().to_vec();
    if decode_b64 {
        signature = general_purpose::STANDARD.decode(&signature)?;
    }

    let signature_len = signature.len();
    let license_blob: Vec<u8> = vec![signature, vec![0; 65 - signature_len], encrypted_data]
        .into_iter()
        .flatten()
        .collect();

    let mut file = File::create(path).unwrap();
    file.write_all(license_blob.as_slice())?;
    file.flush()?;

    Ok(())
}

pub fn save_licenses(license: &License) -> Result<()> {
    let path = get_license_dir()?;

    save_license(&license, path.join(format!("{}.dlf", license.content_id)))?;
    save_license(
        &license,
        path.join(format!("{}_cached.dlf", license.content_id)),
    )?;

    Ok(())
}

#[cfg(windows)]
pub fn get_license_dir() -> Result<PathBuf> {
    let path = format!("C:/{}", LICENSE_PATH.to_string());
    create_dir_all(&path)?;
    Ok(PathBuf::from(path))
}

#[cfg(unix)]
pub fn get_license_dir() -> Result<PathBuf> {
    use crate::unix::wine::wine_prefix_dir;

    let path = format!(
        "{}/drive_c/{}",
        wine_prefix_dir()?.to_str().unwrap(),
        LICENSE_PATH.to_string()
    );
    create_dir_all(&path)?;

    Ok(PathBuf::from(path))
}
