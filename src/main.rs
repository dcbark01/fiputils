mod county;
mod lookup;

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::{fs, process};

use clap::{Parser, Subcommand};

const SHAPEFILE_URL: &str = "https://www.weather.gov/source/gis/Shapefiles/County/c_18mr25.zip";

#[derive(Parser)]
#[command(name = "fiputils")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Look up a FIPS code by state abbreviation and county name.
    Name2fip {
        /// Two-letter state abbreviation (e.g. TX).
        #[arg(long)]
        state: String,
        /// County name (e.g. Travis).
        #[arg(long)]
        county: String,
    },
    /// Look up a state/county name by FIPS code.
    Fip2name {
        /// 5-digit FIPS code (e.g. 48453) or 2-digit state code (e.g. 48).
        code: String,
    },
    /// Look up a FIPS code by lat/lon point.
    Point2fip {
        /// Point as "lat,lon" (e.g. 30.2672,-97.7431).
        #[arg(allow_hyphen_values = true)]
        point: String,
    },
    /// Export all state FIPS codes to a CSV file.
    ExportStates {
        /// Output file path.
        #[arg(short, long, default_value = "fips_states.csv")]
        output: PathBuf,
        /// Include US territories (e.g. Puerto Rico, Guam).
        #[arg(long)]
        include_territories: bool,
    },
    /// Export all county FIPS codes to a CSV file.
    ExportCounties {
        /// Output file path.
        #[arg(short, long, default_value = "fips_counties.csv")]
        output: PathBuf,
        /// Include US territories (e.g. Puerto Rico, Guam).
        #[arg(long)]
        include_territories: bool,
    },
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    process::exit(1);
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("could not determine cache directory")
        .join("fiputils")
}

fn find_shp_file(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|ext| ext == "shp"))
}

fn ensure_shapefile() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = cache_dir();

    if dir.exists()
        && let Some(shp) = find_shp_file(&dir)
    {
        return Ok(shp);
    }

    fs::create_dir_all(&dir)?;
    eprintln!("Downloading county shapefile...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("fiputils/0.1.0")
        .build()?;
    let response = client.get(SHAPEFILE_URL).send()?;
    if !response.status().is_success() {
        return Err(format!("download failed: HTTP {}", response.status()).into());
    }
    let bytes = response.bytes()?;

    let cursor = io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    archive.extract(&dir)?;

    find_shp_file(&dir).ok_or_else(|| "no .shp file found in downloaded archive".into())
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn parse_point(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("expected 'lat,lon', got '{s}'"));
    }
    let lat = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("bad latitude: {e}"))?;
    let lon = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("bad longitude: {e}"))?;
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!("latitude {lat} out of range [-90, 90]"));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(format!("longitude {lon} out of range [-180, 180]"));
    }
    Ok((lat, lon))
}

fn main() {
    let cli = Cli::parse();

    let shp_path = match ensure_shapefile() {
        Ok(p) => p,
        Err(e) => die(&format!("failed to load shapefile: {e}")),
    };

    let counties = match county::load_counties(&shp_path) {
        Ok(c) => c,
        Err(e) => die(&format!("failed to parse shapefile: {e}")),
    };

    match cli.command {
        Command::Name2fip { state, county } => {
            match lookup::name_to_fip(&counties, &state, &county) {
                Some(fips) => println!("{fips}"),
                None => die(&format!(
                    "no county found for --state {state} --county {county}"
                )),
            }
        }
        Command::Fip2name { code } => match lookup::fip_to_name(&counties, &code) {
            Some(name) => println!("{name}"),
            None => die(&format!("no match for FIPS code '{code}'")),
        },
        Command::Point2fip { point } => {
            let (lat, lon) = match parse_point(&point) {
                Ok(p) => p,
                Err(e) => die(&e),
            };
            match lookup::point_to_fip(&counties, lat, lon) {
                Some(fips) => println!("{fips}"),
                None => die("point does not fall within any county"),
            }
        }
        Command::ExportStates {
            output,
            include_territories,
        } => {
            let file = fs::File::create(&output)
                .unwrap_or_else(|e| die(&format!("could not create '{}': {e}", output.display())));
            let mut writer = io::BufWriter::new(file);
            writeln!(writer, "code,name,abbreviation").unwrap_or_else(|e| die(&e.to_string()));
            // Collect one row per state, deduplicated by 2-digit prefix.
            let mut seen = std::collections::HashSet::new();
            let mut count = 0;
            for c in counties
                .iter()
                .filter(|c| include_territories || !county::is_territory(&c.state))
            {
                let state_code = &c.fips[..2];
                if !seen.insert(state_code.to_string()) {
                    continue;
                }
                let state_name = county::state_full_name(&c.state).unwrap_or(&c.state);
                writeln!(
                    writer,
                    "{},{},{}",
                    state_code,
                    csv_field(state_name),
                    csv_field(&c.state)
                )
                .unwrap_or_else(|e| die(&e.to_string()));
                count += 1;
            }
            eprintln!("Exported {count} states to '{}'", output.display());
        }
        Command::ExportCounties {
            output,
            include_territories,
        } => {
            let file = fs::File::create(&output)
                .unwrap_or_else(|e| die(&format!("could not create '{}': {e}", output.display())));
            let mut writer = io::BufWriter::new(file);
            writeln!(writer, "code,name,state,abbreviation")
                .unwrap_or_else(|e| die(&e.to_string()));
            let filtered: Vec<_> = counties
                .iter()
                .filter(|c| include_territories || !county::is_territory(&c.state))
                .collect();
            for c in &filtered {
                let state_name = county::state_full_name(&c.state).unwrap_or(&c.state);
                writeln!(
                    writer,
                    "{},{},{},{}",
                    c.fips,
                    csv_field(&c.county_name),
                    csv_field(state_name),
                    csv_field(&c.state)
                )
                .unwrap_or_else(|e| die(&e.to_string()));
            }
            eprintln!(
                "Exported {} counties to '{}'",
                filtered.len(),
                output.display()
            );
        }
    }
}
