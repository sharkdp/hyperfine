# hyperfine
[![Build Status](https://travis-ci.org/sharkdp/hyperfine.svg?branch=master)](https://travis-ci.org/sharkdp/hyperfine)
[![Build status](https://ci.appveyor.com/api/projects/status/pdqq5frgkcj0smrs?svg=true)](https://ci.appveyor.com/project/sharkdp/hyperfine)
[![Version info](https://img.shields.io/crates/v/hyperfine.svg)](https://crates.io/crates/hyperfine)

A command-line benchmarking tool (*inspired by [bench](https://github.com/Gabriel439/bench)*).

**Demo**: Benchmarking [`fd`](https://github.com/sharkdp/fd) and
[`find`](https://www.gnu.org/software/findutils/):

![hyperfine](https://i.imgur.com/5OqrGWe.gif)

## Features

* Statistical analysis across multiple runs.
* Support for arbitrary shell commands.
* Constant feedback about the benchmark progress and current estimates.
* Warmup runs can be executed before the actual benchmark.
* Cache-clearing commands can be set up before each timing run.
* Statistical outlier detection.
* Export results to various formats: CSV, JSON, Markdown.
* Parameterized benchmarks.
* Cross-platform

## Usage

### Basic benchmark

To run a benchmark, you can simply call `hyperfine <command>...`. The argument(s) can be any
shell command. For example:
``` bash
hyperfine 'sleep 0.3'
```

Hyperfine will automatically determine the number of runs to perform for each command. By default,
it will perform *at least* 10 benchmarking runs. To change this, you can use the `-m`/`--min-runs`
option:
``` bash
hyperfine --min-runs 5 'sleep 0.2' 'sleep 3.2'
```

### I/O-heavy programs

If the program execution time is limited by disk I/O, the benchmarking results can be heavily
influenced by disk caches and whether they are cold or warm.

If you want to run the benchmark on a warm cache, you can use the `-w`/`--warmup` option to perform
a certain number of program executions before the actual benchmark:
``` bash
hyperfine --warmup 3 'grep -R TODO *'
```

Conversely, if you want to run the benchmark for a cold cache, you can use the `-p`/`--prepare`
option to run a special command before *each* timing run. For example, to clear harddisk caches
on Linux, you can run
``` bash
sync; echo 3 | sudo tee /proc/sys/vm/drop_caches
```
To use this specific command with Hyperfine, call `sudo -v` to temporarily gain sudo permissions
and then call:
``` bash
hyperfine --prepare 'sync; echo 3 | sudo tee /proc/sys/vm/drop_caches' 'grep -R TODO *'
```

### Parameterized benchmarks

If you want to run a benchmark where only a single parameter is varied (say, the number of
threads), you can use the `-P`/`--parameter-scan` option and call:
``` bash
hyperfine --prepare 'make clean' --parameter-scan num_threads 1 12 'make -j {num_threads}'
```

### Export results

Hyperfine has multiple options for exporting benchmark results: CSV, JSON, Markdown (see `--help`
text for details). To export results to Markdown, for example, you can use the `--export-markdown`
option that will create tables like this:

| Command | Mean [ms] | Min…Max [ms] |
|:---|---:|---:|
| `find . -iregex '.*[0-9]\.jpg$'` | 506.0 ± 8.1 | 495.4…518.6 |
| `find . -iname '*[0-9].jpg'` | 304.9 ± 3.1 | 299.8…309.3 |
| `fd -HI '.*[0-9]\.jpg$'` | 66.2 ± 5.8 | 62.5…86.3 |

## Installation

### On macOS

Hyperfine can be installed via [Homebrew](https://brew.sh):
```
brew install hyperfine
```

### On Ubuntu

Download the appropriate `.deb` package from the [Release page](https://github.com/sharkdp/hyperfine/releases)
and install it via `dpkg`:
```
wget https://github.com/sharkdp/hyperfine/releases/download/v1.2.0/hyperfine_1.2.0_amd64.deb
sudo dpkg -i hyperfine_1.2.0_amd64.deb
```

### On Arch Linux

On Arch Linux, hyperfine can be installed [from the AUR](https://aur.archlinux.org/packages/hyperfine):
```
yaourt -S hyperfine
```

### On Void Linux

Hyperfine can be installed via xbps

```
xbps-install -S hyperfine
```

### With cargo (Linux, macOS, Windows)

Hyperfine can be installed via [cargo](https://doc.rust-lang.org/cargo/):
```
cargo install hyperfine
```

Make sure that you use Rust 1.24 or higher.

### From binaries (Linux, macOS)

Download the corresponding archive from the [Release page](https://github.com/sharkdp/hyperfine/releases).

## Origin of the name

The name *hyperfine* was chosen in reference to the hyperfine levels of caesium 133 which play a crucial role in the
[definition of our base unit of time](https://en.wikipedia.org/wiki/Second#Based_on_caesium_microwave_atomic_clock)
— the second.
