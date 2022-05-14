## ALG-C Codec

> Currently only ascii characters have been tested

### How to build

The compile and build process relies on the cargo-make, which needs to be pre-installed

- Install cargo-make

```
cargo install --force cargo-make
```

- Execute the following command to build. build and testing.

```
cargo make --makefile cargo-make.toml all-flow
```

## How to run

```
cargo build --release
cd target/release
./algc_codec --input-string "ababcbababaa"
```
