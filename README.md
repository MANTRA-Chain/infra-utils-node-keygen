# Genkey

example
```
genkey -g devnet-1-sentry,devnet-1-archive,devnet-1-validator -n 1
```

## Usage
specify the node per group by `:` to override the global node per group value
```
genkey -g devnet-1-sentry:3,devnet-1-archive,devnet-1-validator:2 -n 1
```