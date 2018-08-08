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

## Issues
Please feel free to open issues if any bugs are encountered or if you would like to add a feature.

## License
MIT
