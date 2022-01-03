
# stdin2file

## Motivation

A kind of tee-like program which writes stdin to file, and
optionally compresses it using multiple threads.
Also just a project-exercise for rust learning.

## Installation

Archives of precompiled binaries are available for linux.

Other way is to clone project and compile it yourself.

## Usage

```sh
stdin2file 1.1
hugruu <h.gruszecki@gmail.com>
Write from stdin to file(s), optionally compresses it using given algorithm

USAGE:
    stdin2file [OPTIONS] --chunk <chunk> --output <output>

OPTIONS:
    -c, --chunk <chunk>            Maximum size of single file size [MiB]
    -e, --execute <execute>        Command to execute (instead of stdin) - CURRENTLY UNSUPPORTED
    -h, --help                     Print help information
    -m, --max-files <max-files>    Number of rotated files
    -o, --output <output>          Output file
    -s, --compress <compress>      Compression algorithm [possible values: xz, gz]
    -V, --version                  Print version information
```

## Examples

Copy stdin to 5 rotating files, each 1 MiB before compression

```sh
command | stdin2file -c 1 -m 5 -o test -s xz
```

Split 10 MiB text file using above settings:

```sh
base64 /dev/urandom | head -c 10000000 | stdin2file -c 1 -m 5 -o test -s xz
```

This will result in 5 files:

```sh
$ ls -la
total 3600
-rw-rw-r-- 1 gruszeck gruszeck 806680 Feb  5 18:14 test.5.xz
-rw-rw-r-- 1 gruszeck gruszeck 806532 Feb  5 18:14 test.6.xz
-rw-rw-r-- 1 gruszeck gruszeck 806952 Feb  5 18:14 test.7.xz
-rw-rw-r-- 1 gruszeck gruszeck 806504 Feb  5 18:14 test.8.xz
-rw-rw-r-- 1 gruszeck gruszeck 433092 Feb  5 18:14 test.9.xz
```

## Possible improvements

* add support for lzma
* make Encoder a trait
* pass command as argument instead of pipe

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
