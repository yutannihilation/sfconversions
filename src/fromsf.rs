//! Conversion from {sf} R objects to geo_types
//!
//! Provides simple conversions between sfg, sfc objects from sf, and
//! geometry primitives from geo_types that can be used with other
//! georust libraries powered by [extendr](https://extendr.github.io/extendr/extendr_api/).
//!
//! Due to the [orphan rule](https://github.com/Ixrec/rust-orphan-rules) conversion
//! directly from extendr `Lists` to geo_types is not possible. For that reason
//! a simple struct `Geom` is implemented with a single field `geom` which contains
//! a geo_types `Geometry` enum.
//!
//! ## Example
//!
//! ```
//! use sfconversions::{sfg_to_geom, geom::Geom};
//!
//! #[extendr]
//! fn extract_sfg(x: Robj) -> String {
//!     sfg_to_geom(x).unwrap().print()
//! }
//! ```

use crate::{vctrs::determine_geoms_class, Geom};
use geo_types::Geometry;
use savvy::{savvy, ListSexp, NullSexp, OwnedListSexp, Sexp};

pub fn sfc_to_rsgeo(x: ListSexp) -> savvy::Result<Sexp> {
    let mut rsgeo = OwnedListSexp::new(x.len(), false)?;

    for (i, (_, obj)) in x.iter().enumerate() {
        rsgeo.set_value(i, sfg_to_rsgeo(obj)?)?;
    }

    // see https://github.com/extendr/extendr/pull/540
    // let rsgeo = x
    //     .into_iter()
    //     .map(|(_, robj)| sfg_to_rsgeo(robj)).collect::<List>();
    let cls = determine_geoms_class(&rsgeo.as_read_only());
    rsgeo.set_class(cls)?;
    rsgeo.into()
}

// These functions are for people who do not want to use rsgeo

/// Given an sfc object, creates a vector of `Option<Geometry>`. NULL geometries are stored
/// as `None` and non-null are `Some(Geometry)`. Match on the result to get the underlying
/// geo-types geometry object or handle null geometry.
pub fn sfc_to_geometry(x: ListSexp) -> Vec<Option<Geometry>> {
    x.values_iter()
        .map(|robj| {
            let geo = sfg_to_geom(robj);
            match geo {
                Ok(g) => Some(g.geom),
                Err(_) => None,
            }
        })
        .collect::<Vec<Option<Geometry>>>()
}

pub fn sfc_to_geoms(x: ListSexp) -> Vec<Option<Geom>> {
    x.values_iter()
        .map(|robj| {
            let geo = sfg_to_geom(robj);
            match geo {
                Ok(g) => Some(g),
                Err(_) => None,
            }
        })
        .collect::<Vec<Option<Geom>>>()
}

/// Falliably takes an extendr `Robj` and returns a `Geom` struct.
/// Supports conversion from `"POINT"`, `"MULTIPOINT"`, `"LINESTRING"`, `"MULTILINESTRING"`,
/// `"POLYGON"`, and `"MULTIPOLYGON"` to their corresponding geo_type primitive.
// `GEOMETRYCOLLECTION` are not supported.
///
/// ```
/// use extendr_api::prelude::*;
/// use extendr_api::Doubles;
/// use sfconversions::sfg_to_geometry;
/// // Create an extendr doubles object and set the appropriate class
/// let dbls = Doubles::from_values([0.0, 10.0])
///     .into_robj()
///     .set_class(["XY", "POINT", "sfg"])
///     .unwrap();
///
/// // convert using `sfg_to_geometry()` and extract the underlyig
/// let geo_primitive = sfg_to_geometry(dbls).geom;
/// ```
///
pub fn sfg_to_geom(x: Sexp) -> savvy::Result<Geom> {
    let geom_sexp = sfg_to_rsgeo(x)?;
    geom_sexp.try_into()
}

use crate::constructors::*;

#[savvy]
pub fn sfg_to_rsgeo(x: Sexp) -> savvy::Result<Sexp> {
    match x.get_class() {
        Some(classes) => match classes.get(1) {
            Some(&"POINT") => geom_point(x.try_into()?),
            Some(&"MULTIPOINT") => geom_multipoint(x.try_into()?),
            Some(&"LINESTRING") => geom_linestring(x.try_into()?),
            Some(&"MULTILINESTRING") => geom_multilinestring(x.try_into()?),
            Some(&"POLYGON") => geom_polygon(x.try_into()?),
            Some(&"MULTIPOLYGON") => geom_multipolygon(x.try_into()?),
            _ => Ok(NullSexp.into()),
        },
        None => Ok(NullSexp.into()),
    }
}
