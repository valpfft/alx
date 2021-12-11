# alx

Helps you with quick searches on olx.pl and allegrolokalnie.pl

<img src="/docs/image.png?raw=true" alt="alx" title="alx">
   
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

```sh
❯ alx --help
alx 0.4.0
Hey Alx'er! Let's find something!

USAGE:
    alx [FLAGS] [OPTIONS] <query>...

FLAGS:
        --export-csv    Exports search result into csv
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
        --max <max_price>    Maximum price
        --min <min_price>    Minimum price

ARGS:
    <query>...    Search query
```
