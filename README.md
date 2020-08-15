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
| `m` | 1 | mode: 0=combo 1=triangle 2=rect 3=ellipse 4=circle 5=rotatedrect 6=beziers 7=rotatedellipse (default 1) |
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

Purr is faster than the original implementation.

Benchmark using `-n 100 -m {0-5}`, see the results below.

> hyperfine --parameter-scan m 0 5 -D 1 './target/release/purr -i ./assets/input.png -o assets/purr.{m}.png -n 100 -m {m}' '~/go/bin/primitive -i ./assets/input.png -o assets/primitive.{m}.png -n 100 -m {m}' --export-json benchmark.json


| Command | Mode | Mean [s] | Min [s] | Max [s] | result |
|:---|---|---:|---:|---:|---:|
|purr|combo| 9.980 ± 0.254|9.503|10.381| ![](./assets/purr.0.png)|
|primitive|combo| 14.564 ± 0.328|14.154|15.036| ![](./assets/primitive.0.png)|
|purr|triangle| 6.927 ± 0.201|6.510|7.136| ![](./assets/purr.1.png)|
|primitive|triangle| 10.577 ± 0.266|10.109|10.879| ![](./assets/primitive.1.png)|
|purr|rect| 6.224 ± 0.087|6.132|6.380| ![](./assets/purr.2.png)|
|primitive|rect| 7.505 ± 0.105|7.388|7.765| ![](./assets/primitive.2.png)|
|purr|ellipse| 12.516 ± 0.098|12.354|12.674| ![](./assets/purr.3.png)|
|primitive|ellipse| 14.643 ± 0.279|14.189|15.193| ![](./assets/primitive.3.png)|
|purr|circle| 15.054 ± 0.236|14.739|15.452| ![](./assets/purr.4.png)|
|primitive|circle| 17.375 ± 0.187|17.156|17.760| ![](./assets/primitive.4.png)|
|purr|rotatedrect| 8.045 ± 0.081|7.938|8.220| ![](./assets/purr.5.png)|
|primitive|rotatedrect| 9.181 ± 0.154|8.922|9.496| ![](./assets/primitive.5.png)|


## TODO

- [ ] More primitives
  - [ ] beziers
  - [ ] rotated ellipse
  - [ ] polygon

