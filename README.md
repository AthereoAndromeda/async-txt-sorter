# large-txt-file-sorter
Just a Rust CLI app that sorts large files alphabetically (somewhat inefficently)

## Why?
Mostly started as a learning project to learn more about Rust and stuff. Also because
I kinda needed one so I decide "hey, why not write one?"

This is a toy project, not really designed to be used in production.

## Performance
It isn't that fast (because single-threaded) for large files (around 100MB+) 
but at least it doesn't blow up your memory. The biggest bottleneck is reading. Might improve on I/O later on.


These values aren't exact, expect fluctuation ofc. It also depends on your machinery.

Machine used:
- CPU: AMD Ryzen 5 5600G
- GPU: Integrated Graphics (whatever my CPU uses)
- Memory: 2x8GB Kingston Fury 3200Mhz
- Storage: Kingston NV1 500GB

`time cargo run -r ./test/text.txt`
```
real    0m0.043s
user    0m0.039s
sys     0m0.004s
```

`time cargo run -r ./rockyou.txt` (133MB)
```
real	0m36.123s
user	0m9.108s
sys	    0m26.883s
```

`time cargo run -r ./realhuman_phill.txt` (683MB)
```
real	2m40.218s
user	0m40.279s
sys	    1m59.447s
```