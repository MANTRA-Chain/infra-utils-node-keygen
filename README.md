# CometBFT Key Generator

This tool generates node keys and peer information for CometBFT nodes.

## Example
```
genkey generate-node-keys -g devnet-1-sentry,devnet-1-archive,devnet-1-validator -n 1
```

## Usage
Specify the node per group by `:` to override the global node per group value:
```
genkey generate-node-keys -g devnet-1-sentry:3,devnet-1-archive,devnet-1-validator:2 -n 1
```

Specify the number of validator keys to generate:
```
genkey generate-priv-validator-keys -d mantra-devnet-1 -v validator -n 5
```

Look up an operator address from a consensus address:
```
# Using mainnet (default)
genkey get-operator-address mantravalcons1...

# Using testnet
genkey get-operator-address -n testnet mantravalcons1...

# Using custom endpoint
genkey get-operator-address -e "https://custom.api/validators" mantravalcons1...

# Using custom prefix (default: mantravalcons)
genkey get-operator-address -p customvalcons customvalcons1...
```

### Commands

- `generate-node-keys`: Generates node keys and peer information.
- `generate-priv-validator-keys`: Generates a [`priv_validator_key.json`] and [`pubkey.json`].
- `get-operator-address`: Looks up the operator address for a given consensus address.

### Options for `generate-node-keys`
- `-d, --directory`: Directory to store the generated keys (default: [`node_keys`] "Go to definition").
- `-g, --group_prefix_list`: Comma-separated list of group prefixes (required).
- `-n, --global_node_per_group`: Number of nodes per group (default: [`2`]).
- `-s, --svc_domain`: Service domain (default: `svc.cluster.local`).
- `-N, --namespace`: Namespace (default: `mantrachain-dukong-nodes`).
- `-p, --port`: Port number (default: [`26656`]).

### Options for `generate-priv-validator-keys`
- `-d, --directory`: Directory to store the generated keys (default: [`val_keys`]).
- `-v, --validator_prefix`: Prefix for validator keys (default: [`v`]).
- `-n, --num`: Number of validator keys to generate (default: [`2`]).

### Options for `get-operator-address`
- `-n, --network`: Network to query (default: `mainnet`, options: `mainnet` or `testnet`).
- `-p, --prefix`: Bech32 prefix for consensus addresses (default: `mantravalcons`).
- `-e, --endpoint`: Override the default API endpoint (optional).
