# Stackprof Blame

Outputs the time spent in matched stack samples with the ability to exclude samples.

## Build

```bash
$ cargo build --release
```

## Usage

```bash
$ stackprof_blame --blame <REGEX> --exclude <REGEX> <STACKPROF PROFILE>
```
