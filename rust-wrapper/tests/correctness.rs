mod common;

use common::{assert_file_backed_report, assert_successful_report, fixture};
use sp800_90b_entropy_assessment::{
    conditioning, iid, non_iid, restart, CommonOptions, ConditioningMode, ConditioningOptions,
    DatasetMode, RestartOptions,
};

#[derive(Clone, Copy)]
struct FixtureCase {
    name: &'static str,
    bits_per_symbol: u8,
    dataset_mode: DatasetMode,
}

fn bundled_fixture_cases() -> &'static [FixtureCase] {
    &[
        FixtureCase {
            name: "biased-random-bits.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "biased-random-bytes.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "data.pi.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "normal.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand1_short.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand4_short.bin",
            bits_per_symbol: 4,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand8_short.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "ringOsc-nist.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "truerand_1bit.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::Conditioned,
        },
        FixtureCase {
            name: "truerand_4bit.bin",
            bits_per_symbol: 4,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "truerand_8bit.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
    ]
}

fn fast_fixture_cases() -> &'static [FixtureCase] {
    &[
        FixtureCase {
            name: "biased-random-bits.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "biased-random-bytes.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "normal.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand1_short.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand4_short.bin",
            bits_per_symbol: 4,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "rand8_short.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "truerand_1bit.bin",
            bits_per_symbol: 1,
            dataset_mode: DatasetMode::Conditioned,
        },
        FixtureCase {
            name: "truerand_4bit.bin",
            bits_per_symbol: 4,
            dataset_mode: DatasetMode::InitialEntropy,
        },
        FixtureCase {
            name: "truerand_8bit.bin",
            bits_per_symbol: 8,
            dataset_mode: DatasetMode::InitialEntropy,
        },
    ]
}

#[test]
fn iid_reports_expected_metadata_for_normal_sample() {
    let sample = fixture("normal.bin");
    let report = iid(
        &sample,
        &CommonOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..CommonOptions::default()
        },
    )
    .expect("iid assessment should succeed");

    assert_file_backed_report(&report);
    assert_eq!(report.iid, Some(true));
    assert_eq!(report.filename.as_deref(), Some(sample.to_string_lossy().as_ref()));
}

#[test]
fn non_iid_reports_expected_metadata_for_normal_sample() {
    let sample = fixture("normal.bin");
    let report = non_iid(
        &sample,
        &CommonOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..CommonOptions::default()
        },
    )
    .expect("non-iid assessment should succeed");

    assert_file_backed_report(&report);
    assert_eq!(report.iid, Some(false));
    assert_eq!(report.filename.as_deref(), Some(sample.to_string_lossy().as_ref()));
}

#[test]
fn iid_conditioned_dataset_mode_runs_on_one_bit_fixture() {
    let sample = fixture("truerand_1bit.bin");
    let report = iid(
        &sample,
        &CommonOptions {
            dataset_mode: DatasetMode::Conditioned,
            bits_per_symbol: Some(1),
            quiet: true,
            ..CommonOptions::default()
        },
    )
    .expect("iid conditioned assessment should succeed");

    assert_file_backed_report(&report);
    assert_eq!(report.iid, Some(true));
}

#[test]
fn iid_runs_on_fast_bundled_nist_fixtures() {
    for case in fast_fixture_cases() {
        let sample = fixture(case.name);
        let report = iid(
            &sample,
            &CommonOptions {
                dataset_mode: case.dataset_mode,
                bits_per_symbol: Some(case.bits_per_symbol),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("iid assessment should succeed for {}: {err}", case.name));

        assert_file_backed_report(&report);
        assert_eq!(
            report.filename.as_deref(),
            Some(sample.to_string_lossy().as_ref()),
            "iid report should reference the input fixture for {}",
            case.name
        );
    }
}

#[test]
#[ignore = "exhaustive iid fixture sweep is slower; run explicitly when validating all bundled data"]
fn iid_runs_on_all_bundled_nist_fixtures() {
    for case in bundled_fixture_cases() {
        let sample = fixture(case.name);
        let report = iid(
            &sample,
            &CommonOptions {
                dataset_mode: case.dataset_mode,
                bits_per_symbol: Some(case.bits_per_symbol),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("iid assessment should succeed for {}: {err}", case.name));

        assert_file_backed_report(&report);
        assert_eq!(
            report.filename.as_deref(),
            Some(sample.to_string_lossy().as_ref()),
            "iid report should reference the input fixture for {}",
            case.name
        );
    }
}

#[test]
#[ignore = "exhaustive non-iid fixture sweep is slower; run explicitly when validating all bundled data"]
fn non_iid_runs_on_all_bundled_nist_fixtures() {
    for case in bundled_fixture_cases() {
        let sample = fixture(case.name);
        let report = non_iid(
            &sample,
            &CommonOptions {
                dataset_mode: case.dataset_mode,
                bits_per_symbol: Some(case.bits_per_symbol),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("non-iid assessment should succeed for {}: {err}", case.name));

        assert_file_backed_report(&report);
        assert_eq!(
            report.filename.as_deref(),
            Some(sample.to_string_lossy().as_ref()),
            "non-iid report should reference the input fixture for {}",
            case.name
        );
        assert_eq!(report.iid, Some(false), "expected non-iid result for {}", case.name);
    }
}

#[test]
fn restart_runs_in_non_iid_mode() {
    let sample = fixture("normal.bin");
    let report = restart(
        &sample,
        &RestartOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..RestartOptions::non_iid(0.5)
        },
    )
    .expect("restart assessment should succeed");

    assert_file_backed_report(&report);
    assert_eq!(report.assessment_type.as_deref(), Some("Restart"));
    assert_eq!(report.iid, Some(false));
}

#[test]
fn conditioning_vetted_mode_returns_conditioning_report() {
    let report = conditioning(ConditioningOptions {
        mode: ConditioningMode::Vetted,
        n_in: 256,
        n_out: 128,
        nw: 4,
        h_in: 0.5,
        h_prime: None,
        verbose: false,
    })
    .expect("conditioning assessment should succeed");

    assert_successful_report(&report);
    assert_eq!(report.assessment_type.as_deref(), Some("Conditioning"));
    assert_eq!(report.iid, Some(false));
}
