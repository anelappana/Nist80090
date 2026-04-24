#![allow(dead_code)]

use sp800_90b_entropy_assessment::AssessmentReport;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust-wrapper should live under the repository root")
        .join("vendor/SP800-90B_EntropyAssessment/bin")
        .join(name)
}

pub fn assert_successful_report(report: &AssessmentReport) {
    assert_eq!(report.error_level, 0);
    assert!(
        !report.test_cases.is_empty(),
        "expected at least one test case in the report"
    );
}

pub fn assert_file_backed_report(report: &AssessmentReport) {
    assert_successful_report(report);
    assert!(
        report.sha256.as_deref().is_some(),
        "expected a sha256 digest in the report"
    );
}

pub fn measure<T, F>(label: &str, f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    eprintln!("{label}: {:?}", elapsed);
    (value, elapsed)
}
