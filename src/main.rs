use anyhow::Result;
use clap::{Command as App, Arg};
use ed25519_consensus::SigningKey;
use rand::prelude::*;
use rand::rngs::adapter::ReseedingRng;
use rand::rngs::OsRng;
use rand_chacha::ChaCha20Core;
use tendermint::private_key::PrivateKey;
use tendermint_config::NodeKey;

fn main() -> Result<()> {
    let matches = App::new("Node Key Generator")
        .version("1.0")
        .about("Generates node keys and peer information")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .value_name("DIRECTORY")
                .default_value("node_keys"),
        )
        .arg(
            Arg::new("group_prefix_list")
                .short('g')
                .long("group_prefix_list")
                .value_name("GROUP_PREFIX_LIST")
                .help("Sets the group prefix list with comma separated values")
                .required(true)
                .default_value(""),
        )
        .arg(
            Arg::new("global_node_per_group")
                .short('n')
                .long("global_node_per_group")
                .value_name("GLOBAL_NODE_PER_GROUP")
                .help("Sets the number of global nodes per group")
                .default_value("2"),
        )
        .arg(
            Arg::new("svc_domain")
                .short('s')
                .long("svc_domain")
                .value_name("SVC_DOMAIN")
                .help("Sets the service domain")
                .default_value("svc.cluster.local"),
        )
        .arg(
            Arg::new("namespace")
                .short('N')
                .long("namespace")
                .value_name("NAMESPACE")
                .help("Sets the namespace")
                .default_value("mantrachain-dukong-nodes"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the port")
                .default_value("26656"),
        )
        .get_matches();

    let directory = matches.get_one::<String>("directory").unwrap();
    let group_prefix_list: Vec<&str> = matches
        .get_one::<String>("group_prefix_list")
        .unwrap()
        .split(',')
        .collect();
    let global_node_per_group = matches.get_one::<String>("global_node_per_group").unwrap().parse::<usize>().unwrap();
    let svc_domain = matches.get_one::<String>("svc_domain").unwrap();
    let namespace = matches.get_one::<String>("namespace").unwrap();
    let port = matches.get_one::<String>("port").unwrap().parse::<u16>().unwrap();

    let mut peers: Vec<String> = vec![];
    let mut secrets: Vec<String> = vec![];

    for n in 0..group_prefix_list.len() {
        // split group_prefix_list[n] in group_prefix and node_per_group if `:` is present
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
            let prng = ChaCha20Core::from_entropy();
            let reseeding_rng = ReseedingRng::new(prng, 0, OsRng);
            let signing_key = SigningKey::new(reseeding_rng);
            let private_key = PrivateKey::from(signing_key);
            let node_key = NodeKey {
                priv_key: private_key,
            };

            peers.push(node_key.node_id().to_string() + "@" + group_prefix + "-p2p-" + &m.to_string() + "." + namespace + "." + svc_domain + ":" + &port.to_string());

            std::fs::create_dir_all(directory)?;
            let file_path = format!("{}/{}", directory, secret_name + ".json");
            std::fs::write(file_path, serde_json::to_string_pretty(&node_key)?)?;
        }
    }

    // iterate over secrets to generate kustomize secret generator
    println!("secretGenerator:");
    for secret in secrets {
        println!("  - name: {}", secret);
        println!("    files:");
        println!("      - node_key.json={}.json", secret);
        println!("    type: Opaque");
    }
    println!("generatorOptions:");
    println!("  disableNameSuffixHash: true");

    // concatenate peers with comma
    println!("\n\npeers:");
    let peers_str = peers.join(",");
    println!("{}", peers_str);
    Ok(())
}