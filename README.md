# alx

Helps you with quick searches on olx & allegro
   
# Install
   
1. Install Rust - https://www.rust-lang.org/tools/install
2. Install or upgrade the package:

``` sh
git clone git@github.com:valpfft/alx.git
cd alx
cargo install --path . --force
```
If it fails to compile and you installed Rust a long time ago, try `rustup update` to update Rust to the latest version.

# Configuration

Allegro integration uses Device Flow described here - https://developer.allegro.pl/en/auth/#device-flow

TLDR:
1. Add an app at https://apps.developer.allegro.pl/
2. Set `ALLEGRO_CLIENT_ID` and `ALLEGRO_CLIENT_SECRET` env variables

