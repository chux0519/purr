# Purr

It's a rust implementation of fogleman's [primitive](https://github.com/fogleman/primitive).

Created at Rusty Days Hackathon, 2020.

> The topic is "Emergent phenomena"("Amaze us with simple rules").

By randomly generating primitive(triangle/ellipse/etc.), to fit a given picture.

The rule is really simple, like hill climbing, we randomly genrate some primitives, then try to find the shortest path to the target image.

For more details about the algorithm, check this out: [Hill Climbing](https://en.wikipedia.org/wiki/Hill_climbing)

## Feature

Simple rule, powerful result

### Usage

> cargo run --release --features=cli --bin=purr  --  -i ./assets/input.png -o output.gif -n 100

most `primitive` flags are supported.

| Flag | Default | Description |
| --- | --- | --- |
| `i` | n/a | input file |
| `o` | n/a | output file |
| `n` | n/a | number of shapes |
| `m` | 1 | mode: 0=combo, 1=triangle, 2=rect, 3=ellipse, 4=circle, 5=rotatedrect |
| `j` | 0 | number of parallel workers (default uses all cores) |
| `r` | 256 | resize large input images to this size before processing |
| `s` | 1024 | output image size |

## Example

using

> -i ./assets/input.png -o output.gif -n 150

| primitive | input image | output image | process |
| --- | --- | --- | --- |
| triangle | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/QTCWJQt.png" /> | <img src="https://i.imgur.com/oGO2rnR.gif" /> |
| ellipse | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/xuWuvs0.png" /> | <img src="https://i.imgur.com/5e4Q8J0.gif" /> |

try it yourself for more.


## About Performance

This program is CPU intensive, it does all rendering in memory.

But it still runs very fast on release build, It's even faster than the original implementation.


| Command | Mean [s] | Min [s] | Max [s] | result |
|:---|---:|---:|---:|---:|
| `./target/release/purr -i ./assets/input.png -o assets/purr.0.png -n 100 -m 0` | 8.185 ± 0.436 | 7.170 | 8.587 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.0.png -n 100 -m 0` | 15.164 ± 0.172 | 14.812 | 15.352 | 1.00 |
| `./target/release/purr -i ./assets/input.png -o assets/purr.1.png -n 100 -m 1` | 3.654 ± 0.108 | 3.537 | 3.804 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.1.png -n 100 -m 1` | 10.610 ± 0.233 | 10.334 | 10.953 | 1.00 |
| `./target/release/purr -i ./assets/input.png -o assets/purr.2.png -n 100 -m 2` | 4.396 ± 0.052 | 4.329 | 4.488 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.2.png -n 100 -m 2` | 7.751 ± 0.190 | 7.490 | 8.040 | 1.00 |
| `./target/release/purr -i ./assets/input.png -o assets/purr.3.png -n 100 -m 3` | 10.907 ± 0.105 | 10.736 | 11.096 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.3.png -n 100 -m 3` | 14.735 ± 0.107 | 14.564 | 14.852 | 1.00 |
| `./target/release/purr -i ./assets/input.png -o assets/purr.4.png -n 100 -m 4` | 13.485 ± 0.172 | 13.158 | 13.735 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.4.png -n 100 -m 4` | 17.508 ± 0.159 | 17.218 | 17.712 | 1.00 |
| `./target/release/purr -i ./assets/input.png -o assets/purr.5.png -n 100 -m 5` | 5.124 ± 0.042 | 5.073 | 5.184 | 1.00 |
| `~/go/bin/primitive -i ./assets/input.png -o assets/primitive.5.png -n 100 -m 5` | 9.250 ± 0.174 | 9.031 | 9.659 | 1.00 |

## TODO

- [ ] More primitives
  - [ ] beziers
  - [ ] rotated ellipse
  - [ ] polygon

