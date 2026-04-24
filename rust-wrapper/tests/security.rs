mod common;

use common::{assert_file_backed_report, fixture};
use sp800_90b_entropy_assessment::{
    conditioning, iid, CommonOptions, ConditioningMode, ConditioningOptions, Error, RestartOptions,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn rejects_invalid_bit_width_before_launching_tools() {
    let err = iid(
        fixture("normal.bin"),
        &CommonOptions {
            bits_per_symbol: Some(9),
            ..CommonOptions::default()
        },
    )
    .expect_err("invalid bit width should fail fast");

    match err {
        Error::InvalidBitsPerSymbol { value } => assert_eq!(value, 9),
        other => panic!("expected InvalidBitsPerSymbol, got {other:?}"),
    }
}

#[test]
fn non_vetted_conditioning_requires_h_prime() {
    let err = conditioning(ConditioningOptions {
        mode: ConditioningMode::NonVetted,
        n_in: 256,
        n_out: 128,
        nw: 4,
        h_in: 0.5,
        h_prime: None,
        verbose: false,
    })
    .expect_err("missing h_prime should fail fast");

    match err {
        Error::MissingArgument { tool, field } => {
            assert_eq!(tool, "ea_conditioning");
            assert_eq!(field, "h_prime");
        }
        other => panic!("expected MissingArgument, got {other:?}"),
    }
}

#[test]
fn missing_input_file_surfaces_as_io_error() {
    let missing = fixture("does-not-exist.bin");
    let err = iid(
        &missing,
        &CommonOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..CommonOptions::default()
        },
    )
    .expect_err("missing file should not succeed");

    match err {
        Error::CommandFailed { .. } | Error::Io { .. } | Error::AssessmentFailed { .. } => {}
        other => panic!("unexpected error variant for missing input: {other:?}"),
    }
}

#[test]
fn hostile_filename_is_treated_as_data_not_shell_syntax() {
    let temp = tempdir().expect("tempdir should be created");
    let hostile_name = "sample;touch hacked && echo owned.bin";
    let hostile_path = temp.path().join(hostile_name);

    fs::copy(fixture("normal.bin"), &hostile_path).expect("fixture should copy");

    let report = iid(
        &hostile_path,
        &CommonOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..CommonOptions::default()
        },
    )
    .expect("hostile filename should still be processed safely");

    assert_file_backed_report(&report);
    assert_eq!(
        report.filename.as_deref(),
        Some(hostile_path.to_string_lossy().as_ref())
    );
    assert!(
        !temp.path().join("hacked").exists(),
        "shell metacharacters in filenames should not trigger command execution"
    );
}

#[test]
fn restart_option_constructor_defaults_are_safe() {
    let options = RestartOptions::iid(0.75);

    assert_eq!(options.bits_per_symbol, None);
    assert_eq!(options.simulation_rounds, None);
    assert_eq!(options.verbose, 0);
    assert!(!options.quiet);
}
