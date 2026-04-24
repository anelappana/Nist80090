use serde_json::Value;
use sp800_90b_entropy_assessment::{non_iid, CommonOptions, DatasetMode};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dataset = env::var("DEMO_DATASET").unwrap_or_else(|_| "data.pi.bin".to_string());
    let bits_per_symbol = env::var("DEMO_BITS")
        .ok()
        .and_then(|bits| bits.parse::<u8>().ok())
        .unwrap_or(8);
    let label = env::var("DEMO_LABEL").unwrap_or_else(|_| "Largest bundled NIST sample".to_string());

    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust-wrapper should live under the repository root")
        .join("vendor/SP800-90B_EntropyAssessment/bin")
        .join(&dataset);

    let options = CommonOptions {
        dataset_mode: DatasetMode::InitialEntropy,
        bits_per_symbol: Some(bits_per_symbol),
        quiet: true,
        ..CommonOptions::default()
    };

    println!("NIST SP 800-90B Entropy Assessment Demo");
    println!("=======================================");
    println!("Scenario   : {label}");
    println!("Input file : {}", sample.display());
    println!("Symbol size: {bits_per_symbol} bits per symbol");
    println!();

    println!("Running non-IID entropy assessment...");
    let non_iid_report = non_iid(&sample, &options)?;
    print_report_summary("Non-IID", &non_iid_report);
    println!();

    println!("Demo takeaway");
    println!("-------------");
    println!(
        "The Rust wrapper successfully built and executed the upstream NIST tool, parsed its JSON output, and exposed the important entropy fields as Rust data."
    );
    println!(
        "For this dataset, the non-IID overall assessed entropy is {} bits per {bits_per_symbol}-bit symbol.",
        format_number(overall_field(&non_iid_report, "hAssessed")),
    );

    Ok(())
}

fn print_report_summary(label: &str, report: &sp800_90b_entropy_assessment::AssessmentReport) {
    println!("{label} result");
    println!("  Tool version : {}", report.tool_version.as_deref().unwrap_or("unknown"));
    println!("  Error level  : {}", report.error_level);
    println!("  SHA-256      : {}", report.sha256.as_deref().unwrap_or("missing"));
    println!("  IID flag     : {}", format_bool(report.iid));
    println!("  hOriginal    : {}", format_number(overall_field(report, "hOriginal")));
    println!("  hAssessed    : {}", format_number(overall_field(report, "hAssessed")));
    println!("  Tests run    : {}", report.test_cases.len());
}

fn overall_field(report: &sp800_90b_entropy_assessment::AssessmentReport, field: &str) -> Option<f64> {
    report
        .test_cases
        .iter()
        .find(|case| case.get("testCaseDesc").and_then(Value::as_str) == Some("Overall"))
        .and_then(|case| case_field(Some(case), field))
}

fn case_field(case: Option<&Value>, field: &str) -> Option<f64> {
    case.and_then(|case| case.get(field)).and_then(Value::as_f64)
}

fn format_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn format_number(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "missing".to_string())
}
