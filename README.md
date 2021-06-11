
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
