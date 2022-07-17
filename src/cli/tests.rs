use std::{ffi::OsString, iter};

use super::{error::Error, try_parse_args, Args};

fn gen_args(args: &[&str]) -> Vec<OsString> {
    iter::once("<EXE>")
        .chain(args.iter().copied())
        .map(OsString::from)
        .collect()
}

#[test]
fn default() {
    let args = try_parse_args(gen_args(&[])).unwrap();
    assert_eq!(args, Args::default());
}

#[test]
fn help() {
    let err = try_parse_args(gen_args(&["--help"])).unwrap_err();
    assert!(matches!(err, Error::DisplayHelp));
}

#[test]
fn version() {
    let err = try_parse_args(gen_args(&["--version"])).unwrap_err();
    assert!(matches!(err, Error::DisplayVersion));
}

#[test]
fn bench() {
    let args = try_parse_args(gen_args(&["--bench"])).unwrap();
    assert_eq!(
        args,
        Args {
            bench: true,
            ..Args::default()
        }
    );
}

#[test]
fn cargo_criterion_filter() {
    let args = try_parse_args(gen_args(&["--bench", "filter"])).unwrap();
    assert_eq!(
        args,
        Args {
            bench: true,
            filter: Some("filter".into()),
            ..Args::default()
        }
    );
}

#[test]
fn cargo_bench_filter() {
    let args = try_parse_args(gen_args(&["filter", "--bench"])).unwrap();
    assert_eq!(
        args,
        Args {
            bench: true,
            filter: Some("filter".into()),
            ..Args::default()
        }
    );
}

#[test]
fn parse_baselines() {
    let args = try_parse_args(gen_args(&[
        "--compare",
        "--baselines",
        "some,list,of,baselines",
    ]))
    .unwrap();
    assert_eq!(
        args,
        Args {
            compare: true,
            baselines: vec![
                "some".into(),
                "list".into(),
                "of".into(),
                "baselines".into()
            ],
            ..Args::default()
        }
    );
}
