# Purr

It's a rust implementation of fogleman's [primitive](https://github.com/fogleman/primitive).

Created at Rusty Days Hackathon, 2020.

For more details about the algorithm, check this out: [Hill Climbing](https://en.wikipedia.org/wiki/Hill_climbing)

## Features

- fast(even faster than the original version), check [purrmitive](./purrmitive/README.md) for more details
- provide both rust lib and c lib (WIP)
- there is also a WIP gui app written in Qt, check [chux0519/purrmitive-qt](https://github.com/chux0519/purrmitive-qt)

### Usage

#### Use `purrmitive` as rust lib

> purrmitive = "\*"

#### or install `purr` as binary

> cargo install --bin purr --features="cli" purrmitive

or

> cd purrmitive && cargo run --release --features=cli --bin=purr  --  -i ./assets/input.png -o output.gif -n 100

most `primitive` flags are supported, it should be a dropin replacement in most cases.

#### or use it as a c lib (WIP)

see [purrmitive-ffi](./purrmitive-ffi/README.md)
