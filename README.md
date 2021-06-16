
## Validator machine calibration utility

To make sure gas cost correlates with actual wall clock time we need to be able to calibrate machine and IO performance
across different hardware and OSes.

This utility measures those properties.

## Usage

Use like this

```bash
  cargo install --git https://github.com/near/calibrator --branch main
  calibrator -i 1000000 -c 1000000
```

## Plots

To measure with wider ranges on input sizes use smth like
```bash
  calibrator \
  --io-range \
100,\
1000,\
100000,\
200000,\
1000000,\
2000000,\
10000000,\
20000000,\
30000000,\
40000000,\
50000000,\
1000000000\
  --cpu-range \
1000,\
10000,\
100000\
  --output 'gnuplot:data.txt'
```

It will produce two files, `io_data.txt` and `cpu_data.txt` which could be plotted with `gnuplot`, i.e.

```bash
    brew install gnuplot
    gnuplot plot.gn
    open perf.png
```
