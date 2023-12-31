# async-txt-sorter
Just a Rust CLI app that sorts large files alphabetically

```bash
Sort large text files quickly

Usage: async-txt-sorter [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to file

Options:
  -o, --output <OUTPUT_PATH>
          Output path. Defaults to res.txt
  -d, --delimiter <DELIMITER>
          Determines on which character to split the file to. Defaults to newline [default: "\n"]
  -D, --output-delimiter <OUTPUT_DELIMITER>
          Determines how the output should be joined together. Defaults to newline [default: "\n"]
  -L, --low-memory-mode
          Lowers memory usage, but takes a lot longer. Disabled by default, but enables if the file is larger than 500MB
  -l, --disable-low-memory-mode
          Disables low memory usage even for files larger than 500MB. Has no effect for files under 500MB
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

## Why?
Mostly started as a learning project to learn more about Rust and stuff. Also because
I kinda needed one so I decide "hey, why not write one?". Then I started to delve deep
and learn more about async Rust and Tokio.

This is a toy project, not really designed to be used in production.

## Modes
- **Standard**  
This mode copies and sorts the entire file in-memory. Pretty quick, but takes as much memory as the given file.

- **Slow (Low-Memory)**  
This mode writes the contents of the file into a temporary directory. It then sorts each file from there. This has less
memory usage, but it is slower since it uses the file system rather than memory.

## Performance
~~It isn't that fast (because single-threaded) for large files (around 100MB+) 
but at least it doesn't blow up your memory. The biggest bottleneck is reading. Might improve on I/O later on.~~

So I replaced it with Tokio and it became pretty fast. Shows how much of a bottleneck I/O really is.
Problem is that all the file's content is just dumped into memory, so it can lead to high memory usage
for very large files. Might change it eventually, dunno yet.

I also used rayon to sort the Vec\<String\> in parallel

These values aren't exact, expect fluctuation ofc. It also depends on your machinery.

Compilation time is not included.

Machine used:
- CPU: AMD Ryzen 5 5600G
- GPU: Integrated Graphics (whatever my CPU uses)
- Memory: 2x8GB Kingston Fury 3200Mhz
- Storage: Kingston NV1 500GB

`time cargo run -r ./test/text.txt`
```
real	0m0.059s
user	0m0.051s
sys     0m0.009s
```

`time cargo run -r ./rockyou.txt` (133MB)
```
real	0m2.377s
user	0m5.039s
sys	    0m0.495s
```

`time cargo run -r ./realhuman_phill.txt` (683MB)
```
real	0m8.751s
user	0m22.529s
sys     0m1.574s
```