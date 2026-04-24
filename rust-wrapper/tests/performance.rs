mod common;

use common::{fixture, measure};
use sp800_90b_entropy_assessment::{
    conditioning, iid, non_iid, CommonOptions, ConditioningMode, ConditioningOptions,
};
use std::time::Duration;

fn assert_reasonable_duration(name: &str, elapsed: Duration) {
    assert!(
        elapsed < Duration::from_secs(30),
        "{name} took too long for the sample fixture: {:?}",
        elapsed
    );
}

#[test]
#[ignore = "performance smoke test; run explicitly when checking wrapper speed"]
fn iid_sample_completes_quickly() {
    let (_, elapsed) = measure("iid normal.bin", || {
        iid(
            fixture("normal.bin"),
            &CommonOptions {
                bits_per_symbol: Some(8),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .expect("iid assessment should succeed")
    });

    assert_reasonable_duration("iid", elapsed);
}

#[test]
#[ignore = "performance smoke test; run explicitly when checking wrapper speed"]
fn non_iid_sample_completes_quickly() {
    let (_, elapsed) = measure("non_iid normal.bin", || {
        non_iid(
            fixture("normal.bin"),
            &CommonOptions {
                bits_per_symbol: Some(8),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .expect("non-iid assessment should succeed")
    });

    assert_reasonable_duration("non_iid", elapsed);
}

#[test]
#[ignore = "performance smoke test; run explicitly when checking wrapper speed"]
fn conditioning_sample_completes_quickly() {
    let (_, elapsed) = measure("conditioning vetted", || {
        conditioning(ConditioningOptions {
            mode: ConditioningMode::Vetted,
            n_in: 256,
            n_out: 128,
            nw: 4,
            h_in: 0.5,
            h_prime: None,
            verbose: false,
        })
        .expect("conditioning assessment should succeed")
    });

    assert_reasonable_duration("conditioning", elapsed);
}
