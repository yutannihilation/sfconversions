//! Conversion from geo-types to {sf} type objects
//!
//! Provides simple conversion from `Geom` wrapper struct to an sfg class object.
//! Additionally provides the ability to convert from `Vec<Option<Geom>>` to a list
//! of sfg objects that can be easily converted into an sfc object by running `sf::st_sfc()`.
//!
use crate::Geom;
/// Takes a single Geom struct and creates the corresponding `sfg` object
use geo_types::*;
use savvy::{NullSexp, OwnedListSexp, OwnedRealSexp, Sexp};

/// A general purpose function that matches on the `Geometry` enum to convert into the
/// appropriate sfg object type. If the Geom cannot be matched (e.g. Line or Triangle),
/// it will return a `NULL` Robj.
pub fn to_sfg(x: Geom) -> savvy::Result<Sexp> {
    let x = x.geom;
    match x {
        Geometry::Point(x) => from_point(x),
        Geometry::MultiPoint(x) => from_multipoint(x),
        Geometry::LineString(x) => from_linestring(x),
        Geometry::MultiLineString(x) => from_multilinestring(x),
        Geometry::Polygon(x) => from_polygon(x),
        Geometry::MultiPolygon(x) => from_multipolygon(x),
        _ => Ok(NullSexp.into()),
    }
}

/// Takes a `Vec<Option<Geom>>` such as the result of `sfc_to_geometry()`
/// and creates a list of sfg objects. This can be easily turned into an `sfc`
/// by passing the results to `sf::st_sfc()`. This cannot be converted into an
/// `sfc` object without first calculating the bounding box which would require
/// importing geo.
pub fn geoms_to_sfc(x: Vec<Option<Geom>>) -> savvy::Result<OwnedListSexp> {
    //let cls = determine_sfc_class(&x).to_ascii_uppercase();
    // let cls_array = [format!("sfc_{cls}"), String::from("sfc")];
    let mut out = OwnedListSexp::new(x.len(), false)?;

    for (i, geom) in x.into_iter().enumerate() {
        if let Some(geo) = geom {
            out.set_value(i, to_sfg(geo)?)?;
        }
    }

    Ok(out)
}

/// Utility function to identify the class of an sfc object .
pub fn determine_sfc_class(x: &Vec<Option<Geom>>) -> String {
    let mut result = String::new();
    for geom in x {
        match geom {
            Some(geom) => {
                let fstr = format!("{:?}", geom.geom);
                let cls = fstr.split('(').next().unwrap().to_string();
                if result.is_empty() {
                    result = cls;
                } else if result != cls {
                    result = "GEOMETRYCOLLECTION".to_string();
                    break;
                }
            }
            None => continue,
        }
    }
    result
}

fn from_coord(x: Coord) -> [f64; 2] {
    [x.x, x.y]
}

/// Convert a `Point` to a sfg
pub fn from_point(x: Point) -> savvy::Result<Sexp> {
    let x = from_coord(x.0);

    let mut out: Sexp = x.as_slice().try_into()?;
    out.set_class(["XY", "POINT", "sfg"])?;
    Ok(out)
}

/// Convert a `MultiPoint` to an sfg
pub fn from_multipoint(x: MultiPoint) -> savvy::Result<Sexp> {
    let x = x
        .into_iter()
        .map(|p| from_coord(p.into()))
        .collect::<Vec<[f64; 2]>>();

    let mut res = new_matrix(x.len(), 2, |r, c| x[r][c])?;
    res.set_class(["XY", "MULTIPOINT", "sfg"])?;
    Ok(res.into())
}

/// Convert a `LineString` to an sfg
pub fn from_linestring(x: LineString) -> savvy::Result<Sexp> {
    let x = x.into_iter().map(from_coord).collect::<Vec<[f64; 2]>>();

    let mut res = new_matrix(x.len(), 2, |r, c| x[r][c])?;
    res.set_class(["XY", "LINESTRING", "sfg"])?;
    Ok(res.into())
}

fn new_matrix(
    nrows: usize,
    ncols: usize,
    f: impl Fn(usize, usize) -> f64,
) -> savvy::Result<OwnedRealSexp> {
    let mut out = OwnedRealSexp::new(nrows * ncols)?;
    for i in 0..nrows {
        for j in 0..ncols {
            let idx = crate::constructors::to_index(i as _, j as _, nrows as _);
            out[idx] = f(i, j);
        }
    }

    Ok(out)
}

/// Convert a `MultiLineString` to an sfg
pub fn from_multilinestring(x: MultiLineString) -> savvy::Result<Sexp> {
    let mut out = OwnedListSexp::new(x.0.len(), false)?;

    for (i, line_string) in x.into_iter().enumerate() {
        out.set_value(i, from_linestring(line_string)?)?;
    }

    out.set_class(["XY", "MULTILINESTRING", "sfg"])?;
    out.into()
}

/// Convert a `Polygon` to an sfg
pub fn from_polygon(x: Polygon) -> savvy::Result<Sexp> {
    let exterior = x.exterior().to_owned();
    let interriors = x.interiors().to_owned();

    // combine the exterior ring and interrior rings into 1 vector first
    // then iterate through them.
    // no method to go from Polygon to multilinestring
    let mut res: Vec<LineString> = Vec::with_capacity(interriors.len() + 1);
    res.push(exterior);
    res.extend(interriors);

    let mut out = OwnedListSexp::new(res.len(), false)?;

    for (i, line_string) in res.into_iter().enumerate() {
        out.set_value(i, from_linestring(line_string)?)?;
    }

    out.set_class(["XY", "POLYGON", "sfg"])?;
    out.into()
}

/// Convert a `MultiPolygon` to an sfg
pub fn from_multipolygon(x: MultiPolygon) -> savvy::Result<Sexp> {
    let mut out = OwnedListSexp::new(x.0.len(), false)?;

    for (i, polygon) in x.into_iter().enumerate() {
        out.set_value(i, from_polygon(polygon)?)?;
    }

    out.set_class(["XY", "MULTIPOLYGON", "sfg"])?;
    out.into()
}
