use std::path::Path;

use geo::{LineString, MultiPolygon, Polygon};
use shapefile::dbase::FieldValue;

pub struct County {
    pub fips: String,
    pub state: String,
    pub county_name: String,
    pub geometry: MultiPolygon<f64>,
}

fn get_string_field(record: &shapefile::dbase::Record, name: &str) -> Result<String, String> {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => Ok(s.trim().to_string()),
        Some(FieldValue::Character(None)) => Err(format!("field '{name}' is null")),
        Some(_) => Err(format!("field '{name}' is not a string")),
        None => Err(format!("field '{name}' not found")),
    }
}

fn ring_to_linestring(ring: &shapefile::PolygonRing<shapefile::Point>) -> LineString<f64> {
    let points = match ring {
        shapefile::PolygonRing::Outer(pts) | shapefile::PolygonRing::Inner(pts) => pts,
    };
    LineString::from(points.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>())
}

fn shape_polygon_to_geo(poly: shapefile::Polygon) -> MultiPolygon<f64> {
    let mut polygons: Vec<Polygon<f64>> = Vec::new();
    let mut exterior: Option<LineString<f64>> = None;
    let mut holes: Vec<LineString<f64>> = Vec::new();

    for ring in poly.rings() {
        let ls = ring_to_linestring(ring);
        match ring {
            shapefile::PolygonRing::Outer(_) => {
                if let Some(ext) = exterior.take() {
                    polygons.push(Polygon::new(ext, std::mem::take(&mut holes)));
                }
                exterior = Some(ls);
            }
            shapefile::PolygonRing::Inner(_) => {
                holes.push(ls);
            }
        }
    }
    if let Some(ext) = exterior {
        polygons.push(Polygon::new(ext, holes));
    }

    MultiPolygon::new(polygons)
}

pub fn load_counties(shp_path: &Path) -> Result<Vec<County>, Box<dyn std::error::Error>> {
    let mut reader = shapefile::Reader::from_path(shp_path)?;
    let mut counties = Vec::new();

    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result?;

        let fips = match get_string_field(&record, "FIPS") {
            Ok(f) => f,
            Err(_) => continue,
        };
        let state = match get_string_field(&record, "STATE") {
            Ok(s) => s,
            Err(_) => continue,
        };
        let county_name = match get_string_field(&record, "COUNTYNAME") {
            Ok(n) => n,
            Err(_) => continue,
        };

        let geometry = match shape {
            shapefile::Shape::Polygon(poly) => shape_polygon_to_geo(poly),
            _ => continue,
        };

        counties.push(County {
            fips,
            state,
            county_name,
            geometry,
        });
    }

    Ok(counties)
}

static STATE_NAMES: &[(&str, &str)] = &[
    ("AL", "Alabama"),
    ("AK", "Alaska"),
    ("AZ", "Arizona"),
    ("AR", "Arkansas"),
    ("CA", "California"),
    ("CO", "Colorado"),
    ("CT", "Connecticut"),
    ("DE", "Delaware"),
    ("DC", "District of Columbia"),
    ("FL", "Florida"),
    ("GA", "Georgia"),
    ("GU", "Guam"),
    ("HI", "Hawaii"),
    ("ID", "Idaho"),
    ("IL", "Illinois"),
    ("IN", "Indiana"),
    ("IA", "Iowa"),
    ("KS", "Kansas"),
    ("KY", "Kentucky"),
    ("LA", "Louisiana"),
    ("ME", "Maine"),
    ("MD", "Maryland"),
    ("MA", "Massachusetts"),
    ("MI", "Michigan"),
    ("MN", "Minnesota"),
    ("MS", "Mississippi"),
    ("MO", "Missouri"),
    ("MT", "Montana"),
    ("NE", "Nebraska"),
    ("NV", "Nevada"),
    ("NH", "New Hampshire"),
    ("NJ", "New Jersey"),
    ("NM", "New Mexico"),
    ("NY", "New York"),
    ("NC", "North Carolina"),
    ("ND", "North Dakota"),
    ("OH", "Ohio"),
    ("OK", "Oklahoma"),
    ("OR", "Oregon"),
    ("PA", "Pennsylvania"),
    ("PR", "Puerto Rico"),
    ("RI", "Rhode Island"),
    ("SC", "South Carolina"),
    ("SD", "South Dakota"),
    ("TN", "Tennessee"),
    ("TX", "Texas"),
    ("UT", "Utah"),
    ("VT", "Vermont"),
    ("VA", "Virginia"),
    ("VI", "Virgin Islands"),
    ("WA", "Washington"),
    ("WV", "West Virginia"),
    ("WI", "Wisconsin"),
    ("WY", "Wyoming"),
    ("AS", "American Samoa"),
    ("MP", "Northern Mariana Islands"),
    ("PW", "Palau"),
    ("FM", "Federated States of Micronesia"),
    ("MH", "Marshall Islands"),
];

const TERRITORIES: &[&str] = &["GU", "PR", "VI", "AS", "MP", "PW", "FM", "MH"];

pub fn is_territory(abbrev: &str) -> bool {
    let upper = abbrev.to_uppercase();
    TERRITORIES.iter().any(|&t| t == upper)
}

pub fn state_full_name(abbrev: &str) -> Option<&'static str> {
    let upper = abbrev.to_uppercase();
    STATE_NAMES
        .iter()
        .find(|(a, _)| *a == upper)
        .map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_full_name_valid() {
        assert_eq!(state_full_name("TX"), Some("Texas"));
        assert_eq!(state_full_name("CA"), Some("California"));
        assert_eq!(state_full_name("tx"), Some("Texas"));
    }

    #[test]
    fn test_state_full_name_invalid() {
        assert_eq!(state_full_name("ZZ"), None);
        assert_eq!(state_full_name(""), None);
    }

    #[test]
    fn test_state_full_name_territories() {
        assert_eq!(state_full_name("DC"), Some("District of Columbia"));
        assert_eq!(state_full_name("PR"), Some("Puerto Rico"));
        assert_eq!(state_full_name("GU"), Some("Guam"));
    }
}
