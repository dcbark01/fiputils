# fiputils

A CLI tool for looking up US county FIPS codes. Supports three lookup modes: name-to-FIPS, FIPS-to-name, and point-to-FIPS (lat/lon).

On first run, the tool downloads the NWS county shapefile and caches it locally.

## Installation

```sh
cargo build --release
# binary at target/release/fiputils
```

## Usage

### County name → FIPS code

```sh
fiputils name2fip --state TX --county Travis
# 48453
```

### FIPS code → name

```sh
# 5-digit county code
fiputils fip2name 48453
# Texas, Travis

# 2-digit state code
fiputils fip2name 48
# Texas
```

### Lat/lon point → FIPS code

```sh
fiputils point2fip 30.2672,-97.7431
# 48453
```

## How it works

On first run, `fiputils` downloads a county boundary shapefile from the National Weather Service and extracts it into the OS cache directory (`~/.cache/fiputils` on Linux/macOS). Subsequent runs use the cached file.

- `name2fip` does a case-insensitive name match against the shapefile records.
- `fip2name` accepts either a 5-digit county FIPS or a 2-digit state FIPS.
- `point2fip` does a geometric point-in-polygon test against county boundaries.
