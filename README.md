# Rusty Days Hackathon

> The topic is "Emergent phenomena"("Amaze us with simple rules").

Purr using some combination of primitives(triangles in this project), to fit pictures.

The rule is really simple, like hill climbing, we randomly genrate some triangles, then try to find the shortest path to the target image.

It's a rust implementation of fogleman's [primitive](https://github.com/fogleman/primitive)


## Feature

Simple rules

### Usage

> cargo build --release
> 
> ./target/release/purr -i ./assets/input.png -o ./output.png

## Example

input image

![input.png](./assets/input.png)

output image

![input.png](./assets/output.png)

process

![output.gif](./assets/out.gif)

## TODO

- [ ] Multithreading support.
- [ ] More primitives


## About Performance

This program is CPU intensive, it does all rendering in memory.

But it still runs very fast on release build, it even might be faster than fogleman's original implementation in single threaded case;



