# Stackprof Blame

A simple regex-based Stackprof aggregator.

Outputs the time spent in call stack samples matching a regular expression with the ability to exclude nested call stack samples. 

For instance, you might want to know how much time is spent in some library but excluding your own business logic.

```bash
$ stackprof_blame --blame 'gems/the_library/' --exclude 'app/my_business_logic/' some_profile.json
```

Note that regular expressions can match both paths and names.

## Build

```bash
$ cargo build --release
```

## Usage

```bash
$ stackprof_blame --blame <REGEX> --exclude <REGEX> <STACKPROF PROFILE>
```
