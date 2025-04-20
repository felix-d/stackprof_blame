# PProf Blame

A simple command-line tool for analyzing pprof profiles with call stack relationship filtering.

## Overview

`pprof_blame` is a Rust-based utility that processes pprof profile files and filters samples based on function relationships in the call stack. It helps you answer questions like:

- How much time is spent in a particular function or library?
- How much time is spent in a function when called by a specific parent?
- How much time is spent in a function excluding certain call paths?

For example, you might want to know how much time is spent in some library but excluding your own business logic:

```bash
$ pprof_blame --file profile.pb.gz --blame 'gems/the_library' --exclude 'app/my_business_logic'
```

## Usage

The basic command format is:

```bash
pprof_blame --file <PROFILE_FILE> --blame <PATTERN> [--parent <PATTERN>] [--exclude <PATTERN>]
```

### Parameters

- `--file`: Path to the profile file (.pb.gz format)
- `--blame`: Regex pattern for functions to blame (required)
- `--parent`: Optional regex pattern for parent functions
- `--exclude`: Optional regex pattern for functions to exclude

### Examples

1. **Basic usage** - Find all samples that match a specific function:

   ```bash
   pprof_blame --file profile.pb.gz --blame 'process_request'
   ```

2. **With parent filter** - Find samples where a function is called by a specific parent:

   ```bash
   pprof_blame --file profile.pb.gz --blame 'process_request' --parent 'handle_connection'
   ```

3. **With exclusion** - Find samples matching a pattern but exclude certain code paths:
   ```bash
   pprof_blame --file profile.pb.gz --blame 'database' --exclude 'cache_lookup'
   ```

## Understanding the Output

The output shows:

- Number of samples matching the blame pattern
- Total value (typically in ms) of those samples
- Percentage relative to parent samples (if --parent is specified) or total samples
- Number of excluded samples (if any)

Example output:

```
32 blamed samples (420 ms) over 128 total samples (2048 ms) (20.51%).
12 samples (156 ms) were excluded.
```

## License

[MIT License](LICENSE)
