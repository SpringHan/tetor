# Tetor

## Warning

This repository is still being developing with a few bugs. The stable version will be released soon.

## Introduction

Tetor is the TErminal ediTOR with syntax highlighting for main file types. And this program is only tested on Linux.

One thing you should know is that this app has little editing ability.
It's just suitable for editing small files temporarily.  
You can see it as [cat](https://en.wikipedia.org/wiki/Cat_(Unix)) with a little editing ability.

The main reason I create this app is that: I don't use Vim, but I use terminal file browser (like [ranger](https://github.com/ranger/ranger)) regularly. So I need a TUI program that can smoothly browse & edit file.

## Installation

Just run:

```shell
cargo build --release
```

And move the executable file in `target/release/` to your `bin` folder.

## TODO

- [ ] Support editing for ANSI Escapes code
