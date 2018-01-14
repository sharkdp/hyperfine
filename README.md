# hyperfine

A command-line benchmarking tool (*inspired by [bench](https://github.com/Gabriel439/bench)*).

``` bash
> hyperfine [OPTIONS] <command>...
```

**Demo**: Benchmarking [`fd`](https://github.com/sharkdp/fd) and [`find`](https://www.gnu.org/software/findutils/):

![hyperfine](https://i.imgur.com/5OqrGWe.gif)

## Features

* Statistical analysis across multiple runs
* Support for arbitrary shell commands
* Constant feedback about the benchmark progress and current estimates
* Warmup runs can be executed before the actual benchmark

## Installation

Hyperfine can be installed via [cargo](https://doc.rust-lang.org/cargo/):
```
cargo install hyperfine
```

## Origin of the name

The name *hyperfine* was chosen in reference to the hyperfine levels of caesium 133 which play a crucial role in the
[definition of our base unit of time](https://en.wikipedia.org/wiki/Second#Based_on_caesium_microwave_atomic_clock)
— the second.
