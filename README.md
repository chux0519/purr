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

> cargo run --release --features=cli --bin=purr  --  -i ./assets/input.png -o output.gif -n 150

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

> -i ./assets/input.png -o output.png -n 150

| primitive | input image | output image | process |
| --- | --- | --- | --- |
| triangle | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/QTCWJQt.png" /> | <img src="https://i.imgur.com/oGO2rnR.gif" /> |
| ellipse | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/xuWuvs0.png" /> | <img src="https://i.imgur.com/5e4Q8J0.gif" /> |


## TODO

- [ ] More primitives
  - [ ] beziers
  - [ ] rotated ellipse
  - [ ] polygon


## About Performance

This program is CPU intensive, it does all rendering in memory.

But it still runs very fast on release build, It's even faster than the original implementation.


