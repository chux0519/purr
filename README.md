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

args:

- `-i`: input image
- `-o`: output image, supported extensions: `.png|.gif|.svg`
- `-n`: number of shapes
- `-s`: optional, shape, could be `triangle`/`ellipse`, default to `triangle`
- `-t`: optional, number of threads, default to `num_cpus::get()`

## Example

using

> -i ./assets/input.png -o output.png -n 150

| input image | output image | process |
| --- | --- | --- |
| <img src="assets/input.png" width="1024"/> | <img src="assets/output.gif.png" /> | <img src="assets/output.gif" /> |


## TODO

- [ ] More primitives


## About Performance

This program is CPU intensive, it does all rendering in memory.

But it still runs very fast on release build.
