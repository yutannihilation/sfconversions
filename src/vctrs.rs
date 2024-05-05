//! Given a `List` of `Geom` pointers, create a `vctrs_vctr`
//!
//! rsgeo uses a list of pointers to `Geom` structs as a representation of a
//! vector. These functions help to create those vctrs with the appropriate class
//! attributes. Note that Geom pointers themselves are not a vector but must be
//! contained in a list with the appropriate `rs_{GEOM}` class to be treated as such.
//! {rsgeo} has `c()` methods so that `c(geom_struct, geom_struct)` will result
//! in a vctrs_vctr of the appropriate type.  
//!
//! Missing geometries are stored as an `extendr_api::NULL` object. Be sure to handle
//! them accordingly.

use std::collections::HashSet;

use savvy::{ListSexp, Sexp};

/// Converts a List of Geom pointers to a {vctrs} vctr
pub fn as_rsgeo_vctr(x: ListSexp, class: &str) -> savvy::Result<Sexp> {
    // TODO
    let mut out = Sexp(x.inner());

    out.set_class(geom_class(class)).unwrap();

    Ok(out)
}

/// Create a `String` array of the vctrs class
pub fn geom_class(cls: &str) -> [String; 4] {
    let cls = cls.to_uppercase();
    let geom_class = "rs_".to_owned() + cls.as_str();

    [
        geom_class,
        String::from("rsgeo"),
        String::from("vctrs_vctr"),
        String::from("list"),
    ]
}

/// From a List, determine the {vctrs} class of the pointer list
pub fn determine_geoms_class(x: &ListSexp) -> [String; 4] {
    let classes: HashSet<&str> = x
        .values_iter()
        .map(|x| *x.get_class().unwrap().first().unwrap())
        .collect();

    let class = if classes.len() > 1 {
        "geometrycollection"
    } else {
        classes.iter().next().unwrap()
    };

    geom_class(class)
}

/// Check if an object is an rsgeo vector
pub fn is_rsgeo(x: &ListSexp) -> bool {
    match x.get_class() {
        Some(cls) => match cls.first() {
            Some(first_cls) => first_cls.starts_with("rs_"),
            None => false,
        },
        None => false,
    }
}

/// Panics if x is not an rsgeo vector
pub fn verify_rsgeo(x: &ListSexp) -> savvy::Result<()> {
    if is_rsgeo(x) {
        Ok(())
    } else {
        Err("`x` must be a Rust geometry type".into())
    }
}

/// Returns the rsgeo vector type such as "point", "linestring", etc
pub fn rsgeo_type(x: &ListSexp) -> savvy::Result<String> {
    let classes = match x.get_class() {
        Some(classes) if !classes.contains(&"rsgeo") => classes,
        _ => {
            return Err("object is not an `rsgeo` vector".into());
        }
    };

    let cls = classes.first().unwrap();

    if !cls.starts_with("rs_") {
        panic!("Object is not an `rsgeo` vector with `rs_` prefix")
    }

    let mut cls = cls.to_string();
    Ok(cls.split_off(3).to_lowercase())
}
