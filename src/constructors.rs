//! Construct geo-types geometry from R objects
//!
//! These function are used to convert R objects into geo-types geometry.
//! These functions mimic the structure of sfg objects from the sf package.
//! Additional quality of life constructors are made available in {rsgeo}.
use crate::Geom;
use geo_types::{
    coord, point, Coord, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use savvy::{ListSexp, RealSexp, Sexp, TypedSexp};

// TODO REMOVE SCALAR CLASSES
/// Create a single `point` from an x and y value.
pub fn geom_point(x: f64, y: f64) -> savvy::Result<Sexp> {
    let mut out: Sexp = Geom::from(Point::new(x, y)).try_into()?;

    out.set_class(["point", "Geom"])?;
    Ok(out)
}

/// Create a single `multipoint` from a 2 dimensional matrix.
pub fn geom_multipoint(x: RealSexp) -> savvy::Result<Sexp> {
    let mpnt = MultiPoint::new(matrix_to_points(x)?);

    let mut out: Sexp = Geom::from(mpnt).try_into()?;
    out.set_class(["multipoint", "Geom"])?;
    Ok(out)
}

/// Create a single `linestring` from a 2 dimensional matrix.
pub fn geom_linestring(x: RealSexp) -> savvy::Result<Sexp> {
    let coords = matrix_to_coords(x)?;
    let lns = LineString::new(coords);

    let mut out: Sexp = Geom::from(lns).try_into()?;
    out.set_class(["linestring", "Geom"])?;
    Ok(out)
}

/// Create a single `multilinestring` from a list of 2 dimensional matrices.
pub fn geom_multilinestring(x: ListSexp) -> savvy::Result<Sexp> {
    let vec_lns = x
        .values_iter()
        .map(|x| Ok(LineString::new(matrix_to_coords(x.try_into()?)?)))
        .collect::<savvy::Result<Vec<LineString>>>()?;

    let mut out: Sexp = Geom::from(MultiLineString::new(vec_lns)).try_into()?;
    out.set_class(["multilinestring", "Geom"])?;
    Ok(out)
}

/// Create a single `polygon` from a list of 2 dimensional matrices.
pub fn geom_polygon(x: ListSexp) -> savvy::Result<Sexp> {
    let n = x.len();

    let mut linestrings: Vec<LineString> = Vec::with_capacity(n);

    let mut iter = x.values_iter();

    let exterior = match iter.next() {
        Some(x) => matrix_to_coords(x.try_into()?),
        None => return Err("Not a matrix".into()),
    }?;
    let exterior = LineString::new(exterior);

    for xi in iter {
        let coords = matrix_to_coords(xi.try_into()?)?;
        let line = LineString::new(coords);
        linestrings.push(line);
    }

    let polygon = Polygon::new(exterior, linestrings);

    let mut out: Sexp = Geom::from(polygon).try_into()?;
    out.set_class(["polygon", "Geom"])?;
    Ok(out)
}

/// Create a single `multipolygon` from a list of lists of 2 dimensional matrices.
pub fn geom_multipolygon(x: ListSexp) -> savvy::Result<Sexp> {
    let res = MultiPolygon::new(
        x.values_iter()
            .map(|x| Ok(polygon_inner(x.try_into()?)?))
            .collect::<savvy::Result<Vec<Polygon>>>()?,
    );

    let mut out: Sexp = Geom::from(res).try_into()?;
    out.set_class(["multipolygon", "Geom"])?;
    Ok(out)
}

// First, I need to take a matrix and convert into coordinates
/// Convert an `RMatrix<f64>` into a vector of `Coords`.
pub fn matrix_to_coords(x: RealSexp) -> savvy::Result<Vec<Coord>> {
    let (nrow, ncol) = match x.get_dim() {
        Some(dim) if dim.len() == 2 => (dim[0], dim[1]),
        _ => {
            return Err("Not a matrix".into());
        }
    };

    if ncol != 2 {
        return Err(
            "Matrix should have only 2 columns for x and y coordinates, respectively.".into(),
        );
    }

    //let n = nrow.clone();
    let mut coords: Vec<Coord> = Vec::with_capacity(nrow as _);

    for i in 0..nrow {
        let x_slice = x.as_slice();
        let crd = coord! {
            x: x_slice[to_index(i, 0, nrow)],
            y: x_slice[to_index(i, 1, nrow)]
        };
        coords.push(crd);
    }
    Ok(coords)
}

#[inline]
fn to_index(i: i32, j: i32, nrow: i32) -> usize {
    (nrow * (j - 1) + i) as _
}

/// Convert an `RMatrix<f64>` into a vector of `Points`. Is
/// used internally to create `MultiPoint`s.
pub fn matrix_to_points(x: RealSexp) -> savvy::Result<Vec<Point>> {
    let (nrow, ncol) = match x.get_dim() {
        Some(dim) if dim.len() == 2 => (dim[0], dim[1]),
        _ => {
            return Err("Not a matrix".into());
        }
    };

    if ncol != 2 {
        return Err(
            "Matrix should have only 2 columns for x and y coordinates, respectively.".into(),
        );
    }

    //let n = nrow.clone();
    let mut coords: Vec<Point> = Vec::with_capacity(nrow as _);

    for i in 0..nrow {
        let x_slice = x.as_slice();
        let crd = point! {
            x: x_slice[to_index(i, 0, nrow)],
            y: x_slice[to_index(i, 1, nrow)]
        };
        coords.push(crd);
    }
    Ok(coords)
}

// utility function to take a list and convert to a Polygon
// will be used to collect into `Vec<Polygon>` and thus into `MultiPolygon`
fn polygon_inner(x: ListSexp) -> savvy::Result<Polygon> {
    let n = x.len();
    let mut linestrings: Vec<LineString> = Vec::with_capacity(n);

    let mut iter = x.values_iter();

    let exterior = match iter.next() {
        Some(x) => matrix_to_coords(x.try_into()?),
        None => return Err("Not a matrix".into()),
    }?;
    let exterior = LineString::new(exterior);

    for xi in iter {
        let coords = matrix_to_coords(xi.try_into()?)?;
        let line = LineString::new(coords);
        linestrings.push(line);
    }

    Ok(Polygon::new(exterior, linestrings))
}
