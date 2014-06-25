rduperemove
===========

An experiment at implementing a [Btrfs](http://btrfs.wiki.kernel.org/) deduplication tool using Rust.

This is work-in-progress, and **NOT** ready for production use. It won't destroy or corrupt data (the kernel won't allow it), but I won't guarantee it will have the undesired side-effects (such as unlinking already-deduped data).

## Goals

* Ease of deploy

  Depend on a minimal set of libraries and tools.

* Reliability

  Error checking and notification. Let the user know what is going on.

* Speed

  Don't keep the disk waiting, saturate the IO bandwidth. Scale well to large-ish (1Gib+) files. Rust's support for concurrency plays big here.

## Non-goals

These are nice-to-have, but are not being actively sought:

* Low memory footprint

  My use case is powerful servers with lots of RAM. On the time x space tradeoff, I want less time.

* Working with older (< 3.12) kernels

  Deduplicating before the same-extent ioctl was significantly harder. I have no intention of supporting that.

## Dependencies

* Links to [libgcrypt](http://directory.fsf.org/wiki/Libgcrypt) to do the hashing work. It's a very common library, needed by Systemd, Gnome, cryptsetup and Xorg, among others.

* Should work on Linux >= 3.12, but I only tested on 3.15.

## Building

The Makefile is from [rust-empty](https://github.com/bvssvni/rust-empty). If you don't have the latest rust nightly yet, you can get it by running `make nightly-install`. Then, just run `make exe` and get your executable at the `bin` folder.

## Acknowledgements

Heavily inspired by [duperemove](https://github.com/markfasheh/duperemove/)
and [bedup](https://github.com/g2p/bedup).
