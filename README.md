![](https://github.com/puripuri2100/llmaker/workflows/build/badge.svg)

llmaker version 0.0.1

# formatcls

Make LL(1) token parser code for Rust


# Install using Cargo

Here is a list of minimally required softwares.

* git
* make
* Rust


## Example

### Install Rust and cargo (Ubuntu)

```sh
curl https://sh.rustup.rs -sSf | sh
```

### Install Rust and cargo (Ubuntu on WSL)

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Rust and cargo (Windows)

Please download [installer](https://www.rust-lang.org/tools/install), and starting installer.

### Build and Install

```sh
git clone https://github.com/puripuri2100/llmaker.git
cd llmaker

make install
```


# Usage of formatcls

Type

```sh
llmaker <input file>
```

or

```sh
llmaker <input file> -o <output file>
```

## Starting out

```sh
make demo
```

If `demo/demo.rs` is created, then the setup has been finished correctly.

---

This software released under [the MIT license](https://github.com/puripuri2100/llmaker/blob/master/LICENSE).

Copyright (c) 2020-2021 Naoki Kaneko (a.k.a. "puripuri2100")
