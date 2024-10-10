use serde::{Serialize, ser};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use ed25519_consensus::SigningKey;
use rand::prelude::*;
use rand::rngs::adapter::ReseedingRng;
use rand::rngs::OsRng;
use rand_chacha::ChaCha20Core;
use tendermint::private_key::PrivateKey;
use tendermint::account;
use tendermint_config::NodeKey;
use tendermint_config::PrivValidatorKey;
use tendermint::public_key::Ed25519 as Ed25519;
use subtle_encoding::base64;

#[derive(Debug, Parser)]
#[clap(name = "cometbft Key Generator", version = "1.0", about = "Generates node keys and peer information")]
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    GenerateNodeKeys(GenerateNodeKeysArgs),
    GeneratePrivValidatorKeys(GeneratePrivValidatorKeysArgs),
}

#[derive(Debug, Args)]
#[clap(about = "short nodekey - Generates node keys and peer information", alias = "nodekey")]
struct GenerateNodeKeysArgs {
    #[clap(short = 'd', long = "directory", default_value = "node_keys")]
    directory: String,
    #[clap(short = 'g', long = "group_prefix_list", required = true, default_value = "")]
    group_prefix_list: String,
    #[clap(short = 'n', long = "global_node_per_group", default_value = "2")]
    global_node_per_group: usize,
    #[clap(short = 's', long = "svc_domain", default_value = "svc.cluster.local")]
    svc_domain: String,
    #[clap(short = 'N', long = "namespace", default_value = "mantrachain-dukong-nodes")]
    namespace: String,
    #[clap(short = 'p', long = "port", default_value = "26656")]
    port: u16,
}

#[derive(Debug, Args)]
#[clap(about = "short valkey - Generates a priv_validator_key.json and pubkey.json", alias = "valkey")]
struct GeneratePrivValidatorKeysArgs {
    #[clap(short = 'd', long = "directory", default_value = "val_keys")]
    directory: String,
    #[clap(short = 'v', long = "validator_prefix", default_value = "v")]
    validator_prefix: String,
    #[clap(short = 'n', long = "num", default_value = "2")]
    num_of_validator_key: usize,
}

fn main() -> Result<()> {
    let app = App::parse();

    match app.command {
        Command::GenerateNodeKeys(args) => generate_node_keys(args),
        Command::GeneratePrivValidatorKeys(args) => generate_priv_validator_key(args),
    }
}

fn generate_node_keys(args: GenerateNodeKeysArgs) -> Result<()> {
    let directory = args.directory;
    let group_prefix_list: Vec<&str> = args.group_prefix_list.split(',').collect();
    let global_node_per_group = args.global_node_per_group;
    let svc_domain = args.svc_domain;
    let namespace = args.namespace;
    let port = args.port;

    let mut peers: Vec<String> = vec![];
    let mut secrets: Vec<String> = vec![];

    for n in 0..group_prefix_list.len() {
        let group_prefix: &str;
        let node_per_group: usize;
        if group_prefix_list[n].contains(':') {
            let parts: Vec<&str> = group_prefix_list[n].split(':').collect();
            group_prefix = parts[0];
            node_per_group = parts[1].parse::<usize>().unwrap();
        } else {
            group_prefix = group_prefix_list[n];
            node_per_group = global_node_per_group;
        }

        for m in 0..node_per_group {
            let secret_name = group_prefix.to_string() + "-node-key-" + &m.to_string();
            secrets.push(secret_name.clone());
            let private_key = create_ed25519_private_key();
            let node_key = NodeKey {
                priv_key: private_key,
            };

            peers.push(node_key.node_id().to_string() + "@" + group_prefix + "-p2p-" + &m.to_string() + "." + &namespace + "." + &svc_domain + ":" + &port.to_string());

            std::fs::create_dir_all(&directory)?;
            let file_path = format!("{}/{}", directory, secret_name + ".json");
            std::fs::write(file_path, serde_json::to_string_pretty(&node_key)?)?;
        }
    }

    println!("secretGenerator:");
    for secret in secrets {
        let file_path = format!("{}/{}", directory, secret.clone() + ".json");
        println!("  - name: {}", secret);
        println!("    files:");
        println!("      - node_key.json={}", file_path);
        println!("    type: Opaque");
    }
    println!("generatorOptions:");
    println!("  disableNameSuffixHash: true");

    println!("\n\npeers:");
    let peers_str = peers.join(",");
    println!("{}", peers_str);
    Ok(())
}

fn generate_priv_validator_key(args: GeneratePrivValidatorKeysArgs) -> Result<()> {
    let directory = args.directory;
    let validator_prefix = args.validator_prefix;
    let num_of_validator_key = args.num_of_validator_key;

    for n in 0..num_of_validator_key {
        let sub_directory = format!("{}/{}", directory, validator_prefix.clone() + &n.to_string());
        std::fs::create_dir_all(&sub_directory)?;
        let output = format!("{}/priv_validator_key.json", sub_directory);

        let private_key = create_ed25519_private_key();
        let public_key = private_key.public_key();
        let address = account::Id::from(public_key);

        let priv_validator_key = PrivValidatorKey {
            address,
            pub_key: public_key,
            priv_key: private_key,
        };
        std::fs::write(&output, serde_json::to_string_pretty(&priv_validator_key)?)?;

        let pubkey = format!("{}/pubkey.txt", sub_directory);
        let cosmos_pubkey = CosmosPublicKey::Ed25519(public_key.ed25519().unwrap());
        let json_string = serde_json::to_string(&cosmos_pubkey)?;
        let formatted_json_string = format!("{:?}", json_string);
        std::fs::write(&pubkey, formatted_json_string)?;
    }
    Ok(())
}

fn create_ed25519_private_key() -> PrivateKey {
    let prng = ChaCha20Core::from_entropy();
    let reseeding_rng = ReseedingRng::new(prng, 0, OsRng);
    let signing_key = SigningKey::new(reseeding_rng);
    PrivateKey::from(signing_key)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(tag = "@type", content = "key")] // match mantrachaind comet show-validator
pub enum CosmosPublicKey {
    /// Ed25519 keys
    #[serde(
        rename = "/cosmos.crypto.ed25519.PubKey",
        serialize_with = "serialize_ed25519_base64",
    )]
    Ed25519(Ed25519),
}

/// Serialize the bytes of an Ed25519 public key as Base64. Used for serializing JSON
fn serialize_ed25519_base64<S>(pk: &Ed25519, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    String::from_utf8(base64::encode(pk.as_bytes()))
        .unwrap()
        .serialize(serializer)
}