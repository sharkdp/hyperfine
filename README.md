# hyperfine

A command-line benchmarking tool (inspired by [bench](https://github.com/Gabriel439/bench)).

Usage
``` bash
> hyperfine 'sleep 0.3' 'sleep 1.7'
```

## Demo
![hyperfine](https://i.imgur.com/iVd5lQ8.gif)

## Features

* Statistical analysis across multiple runs
* Support for arbitrary shell commands
* Visual indication of the benchmark progress
* Warmup runs that are executed before the actual benchmark

## Installation

```
cargo install hyperfine
```

## Origin of the name

The name *hyperfine* was chosen in reference to the hyperfine levels of caesium 133 which play a crucial role in the
[definition of our base unit of time](https://en.wikipedia.org/wiki/Second#Based_on_caesium_microwave_atomic_clock)
— the second.
