# alx

Helps you with quick searches on olx
   
# Install
   
1. Install Rust - https://www.rust-lang.org/tools/install
2. Install or upgrade the package:

``` sh
git clone git@github.com:valpfft/alx.git
cd alx
cargo install --path . --force
```
If it fails to compile and you installed Rust a long time ago, try `rustup update` to update Rust to the latest version.


# Usage

## Help

``` shsh
❯ alx --help
alx 0.2.0
Hey Alx'er! Let's find something!

USAGE:
    alx [FLAGS] [OPTIONS] --query <query>

FLAGS:
        --export-csv    Exports search result into csv
    -h, --help          Prints help information
        --setup         Perform initial setup?
    -V, --version       Prints version information

OPTIONS:
    -c, --config-path <config_path>    Config path. [default: $HOME/.config/alx_config.json]
        --max-price <max_price>        Maximum price
        --min-price <min_price>        Minimum price
    -q, --query <query>                Search query
```

## Search

<img src="/docs/image.png?raw=true" alt="alx" title="alx">
