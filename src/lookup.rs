use geo::{Contains, Point};

use crate::county::{self, County};

pub fn name_to_fip<'a>(counties: &'a [County], state: &str, county_name: &str) -> Option<&'a str> {
    let state_upper = state.to_uppercase();
    let county_upper = county_name.to_uppercase();
    counties
        .iter()
        .find(|c| {
            c.state.to_uppercase() == state_upper && c.county_name.to_uppercase() == county_upper
        })
        .map(|c| c.fips.as_str())
}

pub fn fip_to_name(counties: &[County], fips: &str) -> Option<String> {
    match fips.len() {
        2 => {
            let c = counties.iter().find(|c| c.fips.starts_with(fips))?;
            let state_name = county::state_full_name(&c.state)?;
            Some(state_name.to_string())
        }
        5 => {
            let c = counties.iter().find(|c| c.fips == fips)?;
            let state_name = county::state_full_name(&c.state)?;
            Some(format!("{state_name}, {}", c.county_name))
        }
        _ => None,
    }
}

pub fn point_to_fip<'a>(counties: &'a [County], lat: f64, lon: f64) -> Option<&'a str> {
    let point = Point::new(lon, lat);
    counties
        .iter()
        .find(|c| c.geometry.contains(&point))
        .map(|c| c.fips.as_str())
}

#[cfg(test)]
mod tests {
    use geo::{LineString, MultiPolygon, Polygon};

    use super::*;

    fn make_county(fips: &str, state: &str, name: &str, geometry: MultiPolygon<f64>) -> County {
        County {
            fips: fips.to_string(),
            state: state.to_string(),
            county_name: name.to_string(),
            geometry,
        }
    }

    fn empty_geom() -> MultiPolygon<f64> {
        MultiPolygon::new(vec![])
    }

    /// A square polygon from (0,0) to (1,1) — in lon/lat coords.
    fn unit_square_geom() -> MultiPolygon<f64> {
        let exterior = LineString::from(vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.0, 0.0),
        ]);
        MultiPolygon::new(vec![Polygon::new(exterior, vec![])])
    }

    #[test]
    fn test_name_to_fip_found() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(name_to_fip(&counties, "TX", "Travis"), Some("48453"));
    }

    #[test]
    fn test_name_to_fip_case_insensitive() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(name_to_fip(&counties, "tx", "travis"), Some("48453"));
    }

    #[test]
    fn test_name_to_fip_not_found() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(name_to_fip(&counties, "CA", "Travis"), None);
    }

    #[test]
    fn test_fip_to_name_5digit() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(
            fip_to_name(&counties, "48453"),
            Some("Texas, Travis".to_string())
        );
    }

    #[test]
    fn test_fip_to_name_2digit() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(fip_to_name(&counties, "48"), Some("Texas".to_string()));
    }

    #[test]
    fn test_fip_to_name_not_found() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(fip_to_name(&counties, "99999"), None);
    }

    #[test]
    fn test_fip_to_name_bad_length() {
        let counties = vec![make_county("48453", "TX", "Travis", empty_geom())];
        assert_eq!(fip_to_name(&counties, "123"), None);
    }

    #[test]
    fn test_point_to_fip_inside() {
        let counties = vec![make_county("48453", "TX", "Travis", unit_square_geom())];
        assert_eq!(point_to_fip(&counties, 0.5, 0.5), Some("48453"));
    }

    #[test]
    fn test_point_to_fip_outside() {
        let counties = vec![make_county("48453", "TX", "Travis", unit_square_geom())];
        assert_eq!(point_to_fip(&counties, 5.0, 5.0), None);
    }
}
