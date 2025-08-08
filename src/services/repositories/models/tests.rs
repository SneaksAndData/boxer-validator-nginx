use super::http_method::HTTPMethod::Get;
use super::path_segment::PathSegment;
use super::path_segment::PathSegment::{Parameter, Static};
use super::request_segment::RequestSegment;
use super::request_segment::RequestSegment::{Hostname, Path, Verb};

use rstest::rstest;
use std::cmp::Ordering;

#[rstest]
fn test_request_segment_reflexivity(
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    x: RequestSegment,
) {
    assert_eq!(x.cmp(&x), Ordering::Equal);
}

#[rstest]
fn test_request_segment_symmetry(
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    x: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    y: RequestSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        Ordering::Less => assert_eq!(y.cmp(&x), Ordering::Greater),
        Ordering::Equal => assert_eq!(y.cmp(&x), Ordering::Equal),
        Ordering::Greater => assert_eq!(y.cmp(&x), Ordering::Less),
    }
}

#[rstest]
fn test_request_segment_transitivity(
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    x: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    y: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    z: RequestSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        // if x < y and y < z, then x < z
        // if x < y and y == z, then x < z
        Ordering::Less => match y.cmp(&z) {
            Ordering::Less => assert_eq!(x.cmp(&z), Ordering::Less),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Less),
            _ => assert_eq!(true, true), // Skip this case if y > z, we cannot guarantee x < z
        },

        // if x == y and y == z, then x == z
        Ordering::Equal => match y.cmp(&z) {
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Equal),
            _ => assert_eq!(true, x.cmp(&z) == Ordering::Greater || x.cmp(&z) == Ordering::Less),
        },

        // if x > y and y > z, then x > z
        // if x > y and y == z, then x > z
        Ordering::Greater => match y.cmp(&z) {
            Ordering::Greater => assert_eq!(x.cmp(&z), Ordering::Greater),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Greater),
            _ => assert_eq!(true, true), // Skip this case if y < z, we cannot guarantee x > z
        },
    }
}

#[rstest]
fn test_path_segment_reflexivity(
    #[values(Verb(Get), Hostname("example.com".to_string()), Path(Static("api".to_string())), Path(Parameter))]
    x: RequestSegment,
) {
    assert_eq!(x.cmp(&x), Ordering::Equal);
}

#[rstest]
fn test_path_segment_symmetry(
    #[values(Static("api".to_string()), Parameter)] x: PathSegment,
    #[values(Static("api".to_string()), Parameter)] y: PathSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        Ordering::Less => assert_eq!(y.cmp(&x), Ordering::Greater),
        Ordering::Equal => assert_eq!(y.cmp(&x), Ordering::Equal),
        Ordering::Greater => assert_eq!(y.cmp(&x), Ordering::Less),
    }
}

#[rstest]
fn est_path_segment_transitivity(
    #[values(Static("api".to_string()), Parameter)] x: PathSegment,
    #[values(Static("api".to_string()), Parameter)] y: PathSegment,
    #[values(Static("api".to_string()), Parameter)] z: PathSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        // if x < y and y < z, then x < z
        // if x < y and y == z, then x < z
        Ordering::Less => match y.cmp(&z) {
            Ordering::Less => assert_eq!(x.cmp(&z), Ordering::Less),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Less),
            _ => assert_eq!(true, true), // Skip this case if y > z, we cannot guarantee x < z
        },

        // if x == y and y == z, then x == z
        Ordering::Equal => match y.cmp(&z) {
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Equal),
            _ => assert_eq!(true, x.cmp(&z) == Ordering::Greater || x.cmp(&z) == Ordering::Less),
        },

        // if x > y and y > z, then x > z
        // if x > y and y == z, then x > z
        Ordering::Greater => match y.cmp(&z) {
            Ordering::Greater => assert_eq!(x.cmp(&z), Ordering::Greater),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Greater),
            _ => assert_eq!(true, true), // Skip this case if y < z, we cannot guarantee x > z
        },
    }
}
