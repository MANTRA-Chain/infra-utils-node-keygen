use anyhow::Result;
use bech32::{self, Hrp};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Default)]
pub enum Network {
    #[default]
    Mainnet,
    Testnet,
}

impl Network {
    pub fn api_endpoint(&self) -> &str {
        match self {
            Network::Mainnet => "https://api.mantrachain.io/cosmos/staking/v1beta1/validators",
            Network::Testnet => {
                "https://api.dukong.mantrachain.io/cosmos/staking/v1beta1/validators"
            }
        }
    }
}

pub async fn get_operator_address(
    endpoint: &str,
    consensus_addr: &str,
    hrp: &str,
) -> Result<String> {
    let response = reqwest::get(endpoint).await?;
    let json: Value = response.json().await?;

    if let Some(validators) = json["validators"].as_array() {
        for validator in validators {
            if let (Some(op_addr), Some(pubkey_obj)) = (
                validator["operator_address"].as_str(),
                validator["consensus_pubkey"].as_object(),
            ) {
                if let Some(base64_key) = pubkey_obj["key"].as_str() {
                    let pubkey = base64_to_pubkey(base64_key)?;
                    let derived_cons_addr = pubkey_to_consensus_address(pubkey, hrp)?;

                    if derived_cons_addr == consensus_addr {
                        return Ok(op_addr.to_string());
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("No matching validator found"))
}

pub fn pubkey_to_consensus_address(
    pubkey: tendermint::public_key::PublicKey,
    hrp: &str,
) -> Result<String> {
    let pubkey_bytes = pubkey.to_bytes();
    let hash = Sha256::digest(pubkey_bytes);
    let truncated_address = &hash[..20];

    let bech32_address = bech32::encode::<bech32::Bech32>(Hrp::parse(hrp)?, truncated_address)
        .expect("Bech32 encoding failed");
    Ok(bech32_address)
}

pub fn base64_to_pubkey(base64: &str) -> Result<tendermint::public_key::PublicKey> {
    let bytes = subtle_encoding::base64::decode(base64)?;
    let pubkey = tendermint::public_key::PublicKey::from_raw_ed25519(&bytes)
        .ok_or_else(|| anyhow::anyhow!("Invalid public key bytes"))?;
    Ok(pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_operator_address() {
        let mut mock_server = mockito::Server::new_async().await;

        // Create mock response data
        let mock_response = r#"{
            "validators": [
                {
                    "operator_address": "mantravaloper1qe8uuf5x69c526h4nzxwv4ltftr73v7q090nhn",
                    "consensus_pubkey": {
                        "@type": "/cosmos.crypto.ed25519.PubKey",
                        "key": "RLcrEWwJ1kK8uVoMZ+fVac7/hjsZtUZe69Rhzklm5ro="
                    }
                }
            ]
        }"#;

        // Setup mock server
        let mock = mock_server
            .mock("GET", "/cosmos/staking/v1beta1/validators")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Generate the consensus address we want to search for
        let base64_key = "RLcrEWwJ1kK8uVoMZ+fVac7/hjsZtUZe69Rhzklm5ro=";
        let pubkey = base64_to_pubkey(base64_key).unwrap();
        let consensus_addr = pubkey_to_consensus_address(pubkey, "mantravalcons").unwrap();

        // Test the function
        let result = get_operator_address(
            &format!("{}/cosmos/staking/v1beta1/validators", mock_server.url()),
            &consensus_addr,
            "mantravalcons",
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "mantravaloper1qe8uuf5x69c526h4nzxwv4ltftr73v7q090nhn"
        );
    }

    #[tokio::test]
    async fn test_get_operator_address_not_found() {
        let mut mock_server = mockito::Server::new_async().await;

        // Create mock response with no matching validator
        let mock_response = r#"{
            "validators": [
                {
                    "operator_address": "mantravaloper1different",
                    "consensus_pubkey": {
                        "@type": "/cosmos.crypto.ed25519.PubKey",
                        "key": "95GhTICM5hTEUUjBXxtY2UdHD0zz8O2i7ePs3VVME2g="
                    }
                }
            ]
        }"#;

        // Setup mock server
        let mock = mock_server
            .mock("GET", "/cosmos/staking/v1beta1/validators")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test with a consensus address that won't be found
        let result = get_operator_address(
            &format!("{}/cosmos/staking/v1beta1/validators", mock_server.url()),
            "mantravalcons1nonexistent",
            "mantravalcons",
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No matching validator found"
        );
    }
}
