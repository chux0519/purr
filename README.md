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

| primitive | input image | output image | process |
| --- | --- | --- | --- |
| triangle | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/QTCWJQt.png" /> | <img src="https://i.imgur.com/oGO2rnR.gif" /> |
| ellipse | <img src="assets/input.png" width="1024"/> | <img src="https://i.imgur.com/xuWuvs0.png" /> | <img src="https://i.imgur.com/5e4Q8J0.gif" /> |


## TODO

- [ ] More primitives


## About Performance

This program is CPU intensive, it does all rendering in memory.

But it still runs very fast on release build, It's even faster than the original implementation.

using 150 triangles to fit the same picture, following is the run time on my machine bettween `purr` and `primitive`.

CPU: Intel i7-4710MQ (8) @ 3.500GHz

```bash
❯ time ~/go/bin/primitive  -i ./assets/input.png -o /tmp/primitive.out.png -n 150 -s 800

real    0m12.498s
user    1m13.329s
sys     0m0.161s

❯ time ./target/release/purr -i ./assets/input.png -o /tmp/purr.out.png -n 150
Batch: 1, score 0.14954229709897457 -> score 0.1457404023123953
Batch: 2, score 0.1457404023123953 -> score 0.143932220289824
Batch: 3, score 0.143932220289824 -> score 0.14175475487878508
...
real    0m6.273s
user    0m44.507s
sys     0m0.084s
```

150 ellipse

```bash
❯ time ~/go/bin/primitive  -i ./assets/input.png -o /tmp/primitive.out.png -n 150 -s 800 -m 3

real    0m18.274s
user    2m3.772s
sys     0m0.193s

❯ time ./target/release/purr -i ./assets/input.png -o /tmp/purr.out.png -n 150 -s ellipse
Batch: 1, score 0.14954229709897457 -> score 0.14201926628424674
Batch: 2, score 0.14201926628424674 -> score 0.1386901698440043
Batch: 3, score 0.1386901698440043 -> score 0.13543792376356723
...
real    0m16.116s
user    1m57.020s
sys     0m0.086s
```
