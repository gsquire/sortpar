# sortpar

`sortpar` is a command line tool that can sort text files in parallel. It does this by taking
advantage of the [rayon](https://github.com/rayon-rs/rayon) crate's implementation on slices.

[![Build Status](https://travis-ci.org/gsquire/sortpar.svg?branch=master)](https://travis-ci.org/gsquire/sortpar)

## Installation
You can install `sortpar` by running:

```sh
cargo install sortpar
```

This will place a binary called `sp` in the `$HOME/.cargo/bin` directory on your machine. You can
add that path to the `$PATH` variable to have easier access. There are plans to have more
installation options if the tool becomes useful to those without Rust on their system.

## Usage
Currently `sortpar` supports a subset of the options that the Unix sort command does. They can be
listed by running:

```sh
sp --help
```

## Benchmarks
It would be great to have more benchmarks but it is also hard to get an accurate measure across
multiple cases. Just to give an idea of the performance at the moment, I sorted Peter Norvig's
[big text file](https://norvig.com/big.txt). **WARNING**, the link leads to a 6.2MB file.

Using [hyperfine](https://github.com/sharkdp/hyperfine) I got these results:

```sh
Benchmark #1: sp big.txt

  Time (mean ± σ):     445.1 ms ±   7.6 ms    [User: 857.0 ms, System: 90.8 ms]

  Range (min … max):   436.6 ms … 457.4 ms
```

```sh
Benchmark #1: gsort --parallel=4 big.txt

  Time (mean ± σ):      2.604 s ±  0.023 s    [User: 2.550 s, System: 0.032 s]

  Range (min … max):    2.558 s …  2.632 s
```

Why didn't I use `LC_ALL=C` for the GNU sort benchmark? Because it would be unfair to allow GNU
sort to avoid the overhead of UTF-8 decoding. Perhaps in the future `sortpar` can have an option
to do this as well.

## Issues
Please feel free to open issues if any bugs are encountered or if you would like to add a feature.

## License
MIT
