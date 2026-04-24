use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use thiserror::Error;

const BIN_DIR: &str = concat!(env!("OUT_DIR"), "/bin");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetMode {
    InitialEntropy,
    Conditioned,
}

impl DatasetMode {
    fn flag(self) -> &'static str {
        match self {
            Self::InitialEntropy => "-i",
            Self::Conditioned => "-c",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitstringMode {
    UseAllData,
    TruncateToOneMillionBits,
}

impl BitstringMode {
    fn flag(self) -> &'static str {
        match self {
            Self::UseAllData => "-a",
            Self::TruncateToOneMillionBits => "-t",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartMode {
    Iid,
    NonIid,
}

impl RestartMode {
    fn flag(self) -> &'static str {
        match self {
            Self::Iid => "-i",
            Self::NonIid => "-n",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditioningMode {
    Vetted,
    NonVetted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subset {
    pub index: u64,
    pub samples: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonOptions {
    pub dataset_mode: DatasetMode,
    pub bitstring_mode: BitstringMode,
    pub bits_per_symbol: Option<u8>,
    pub subset: Option<Subset>,
    pub verbose: u8,
    pub quiet: bool,
}

impl Default for CommonOptions {
    fn default() -> Self {
        Self {
            dataset_mode: DatasetMode::InitialEntropy,
            bitstring_mode: BitstringMode::UseAllData,
            bits_per_symbol: None,
            subset: None,
            verbose: 0,
            quiet: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartOptions {
    pub mode: RestartMode,
    pub bits_per_symbol: Option<u8>,
    pub h_i: f64,
    pub simulation_rounds: Option<u64>,
    pub verbose: u8,
    pub quiet: bool,
}

impl RestartOptions {
    pub fn non_iid(h_i: f64) -> Self {
        Self {
            mode: RestartMode::NonIid,
            bits_per_symbol: None,
            h_i,
            simulation_rounds: None,
            verbose: 0,
            quiet: false,
        }
    }

    pub fn iid(h_i: f64) -> Self {
        Self {
            mode: RestartMode::Iid,
            ..Self::non_iid(h_i)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConditioningOptions {
    pub mode: ConditioningMode,
    pub n_in: u64,
    pub n_out: u64,
    pub nw: u64,
    pub h_in: f64,
    pub h_prime: Option<f64>,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentReport {
    #[serde(rename = "dateTimeStamp")]
    pub date_time_stamp: Option<String>,
    pub commandline: Option<String>,
    #[serde(rename = "errorLevel")]
    pub error_level: i32,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "type")]
    pub assessment_type: Option<String>,
    #[serde(rename = "toolVersion")]
    pub tool_version: Option<String>,
    pub filename: Option<String>,
    pub sha256: Option<String>,
    #[serde(rename = "IID")]
    pub iid: Option<bool>,
    #[serde(rename = "testCases", default, deserialize_with = "deserialize_test_cases")]
    pub test_cases: Vec<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn deserialize_test_cases<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Vec<Value>>::deserialize(deserializer).map(|cases| cases.unwrap_or_default())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid bits_per_symbol {value}; expected a value in 1..=8")]
    InvalidBitsPerSymbol { value: u8 },
    #[error("missing compiled assessment tool at {path}")]
    MissingBinary { path: PathBuf },
    #[error("{tool} requires the field `{field}`")]
    MissingArgument {
        tool: &'static str,
        field: &'static str,
    },
    #[error("failed to run {tool}")]
    Io {
        tool: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode JSON emitted by {tool}")]
    Json {
        tool: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("{tool} returned an unsuccessful exit status")]
    CommandFailed {
        tool: &'static str,
        status: Option<i32>,
        stdout: String,
        stderr: String,
        report: Option<Box<AssessmentReport>>,
    },
    #[error("{tool} reported an assessment error: {message}")]
    AssessmentFailed {
        tool: &'static str,
        message: String,
        report: Box<AssessmentReport>,
    },
}

pub fn iid<P: AsRef<Path>>(input: P, options: &CommonOptions) -> Result<AssessmentReport, Error> {
    let mut args = common_args(options)?;
    args.push(input.as_ref().as_os_str().to_os_string());
    if let Some(bits) = options.bits_per_symbol {
        args.push(bits.to_string().into());
    }
    run_tool("ea_iid", args)
}

pub fn non_iid<P: AsRef<Path>>(
    input: P,
    options: &CommonOptions,
) -> Result<AssessmentReport, Error> {
    let mut args = common_args(options)?;
    args.push(input.as_ref().as_os_str().to_os_string());
    if let Some(bits) = options.bits_per_symbol {
        args.push(bits.to_string().into());
    }
    run_tool("ea_non_iid", args)
}

pub fn restart<P: AsRef<Path>>(
    input: P,
    options: &RestartOptions,
) -> Result<AssessmentReport, Error> {
    if let Some(bits) = options.bits_per_symbol {
        validate_bits(bits)?;
    }

    let mut args = Vec::new();
    args.push(options.mode.flag().into());

    for _ in 0..options.verbose {
        args.push("-v".into());
    }
    if options.quiet {
        args.push("-q".into());
    }
    if let Some(rounds) = options.simulation_rounds {
        args.push("-s".into());
        args.push(rounds.to_string().into());
    }

    args.push(input.as_ref().as_os_str().to_os_string());
    if let Some(bits) = options.bits_per_symbol {
        args.push(bits.to_string().into());
    }
    args.push(options.h_i.to_string().into());

    run_tool("ea_restart", args)
}

pub fn conditioning(options: ConditioningOptions) -> Result<AssessmentReport, Error> {
    let mut args = Vec::new();
    if options.verbose {
        args.push("-v".into());
    }

    match options.mode {
        ConditioningMode::Vetted => {
            args.push(options.n_in.to_string().into());
            args.push(options.n_out.to_string().into());
            args.push(options.nw.to_string().into());
            args.push(options.h_in.to_string().into());
        }
        ConditioningMode::NonVetted => {
            args.push("-n".into());
            args.push(options.n_in.to_string().into());
            args.push(options.n_out.to_string().into());
            args.push(options.nw.to_string().into());
            args.push(options.h_in.to_string().into());
            let h_prime = options.h_prime.ok_or(Error::MissingArgument {
                tool: "ea_conditioning",
                field: "h_prime",
            })?;
            args.push(h_prime.to_string().into());
        }
    }

    run_tool("ea_conditioning", args)
}

fn common_args(options: &CommonOptions) -> Result<Vec<std::ffi::OsString>, Error> {
    if let Some(bits) = options.bits_per_symbol {
        validate_bits(bits)?;
    }

    let mut args = Vec::new();
    args.push(options.dataset_mode.flag().into());
    args.push(options.bitstring_mode.flag().into());

    for _ in 0..options.verbose {
        args.push("-v".into());
    }
    if options.quiet {
        args.push("-q".into());
    }
    if let Some(subset) = options.subset {
        args.push("-l".into());
        args.push(format!("{},{}", subset.index, subset.samples).into());
    }

    Ok(args)
}

fn validate_bits(bits: u8) -> Result<(), Error> {
    if (1..=8).contains(&bits) {
        Ok(())
    } else {
        Err(Error::InvalidBitsPerSymbol { value: bits })
    }
}

fn run_tool(
    tool: &'static str,
    mut args: Vec<std::ffi::OsString>,
) -> Result<AssessmentReport, Error> {
    let binary = Path::new(BIN_DIR).join(tool);
    if !binary.exists() {
        return Err(Error::MissingBinary { path: binary });
    }

    let json_output = NamedTempFile::new().map_err(|source| Error::Io { tool, source })?;
    args.insert(0, json_output.path().as_os_str().to_os_string());
    args.insert(0, "-o".into());

    let output = Command::new(&binary)
        .args(&args)
        .output()
        .map_err(|source| Error::Io { tool, source })?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let report_text = std::fs::read_to_string(json_output.path()).unwrap_or_default();
    let report = if report_text.trim().is_empty() {
        None
    } else {
        Some(
            serde_json::from_str::<AssessmentReport>(&report_text)
                .map_err(|source| Error::Json { tool, source })?,
        )
    };

    if !output.status.success() {
        return Err(Error::CommandFailed {
            tool,
            status: output.status.code(),
            stdout,
            stderr,
            report: report.map(Box::new),
        });
    }

    let report = report.ok_or_else(|| Error::CommandFailed {
        tool,
        status: output.status.code(),
        stdout,
        stderr,
        report: None,
    })?;

    if report.error_level != 0 {
        return Err(Error::AssessmentFailed {
            tool,
            message: report
                .error_message
                .clone()
                .unwrap_or_else(|| "unknown assessment error".to_string()),
            report: Box::new(report),
        });
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("rust-wrapper should live under the repository root")
            .join("vendor/SP800-90B_EntropyAssessment/bin")
            .join(name)
    }

    #[test]
    fn validates_bit_width() {
        assert!(validate_bits(1).is_ok());
        assert!(validate_bits(8).is_ok());
        assert!(validate_bits(0).is_err());
        assert!(validate_bits(9).is_err());
    }

    #[test]
    fn builds_subset_flag() {
        let options = CommonOptions {
            subset: Some(Subset {
                index: 2,
                samples: 4096,
            }),
            ..CommonOptions::default()
        };

        let args = common_args(&options).expect("args should build");
        let rendered: Vec<String> = args
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(rendered.contains(&"-l".to_string()));
        assert!(rendered.contains(&"2,4096".to_string()));
    }

    #[test]
    fn iid_wrapper_runs_against_sample_data() {
        let report = iid(
            fixture("normal.bin"),
            &CommonOptions {
                bits_per_symbol: Some(8),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .expect("iid wrapper should succeed");

        assert_eq!(report.error_level, 0);
        assert_eq!(report.iid, Some(true));
        assert!(!report.test_cases.is_empty());
        assert_eq!(
            report.filename.as_deref(),
            Some(fixture("normal.bin").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn non_iid_wrapper_runs_against_sample_data() {
        let report = non_iid(
            fixture("normal.bin"),
            &CommonOptions {
                bits_per_symbol: Some(8),
                quiet: true,
                ..CommonOptions::default()
            },
        )
        .expect("non-iid wrapper should succeed");

        assert_eq!(report.error_level, 0);
        assert_eq!(report.iid, Some(false));
        assert!(!report.test_cases.is_empty());
        assert_eq!(
            report.filename.as_deref(),
            Some(fixture("normal.bin").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn conditioning_wrapper_runs_in_vetted_mode() {
        let report = conditioning(ConditioningOptions {
            mode: ConditioningMode::Vetted,
            n_in: 256,
            n_out: 128,
            nw: 4,
            h_in: 0.5,
            h_prime: None,
            verbose: false,
        })
        .expect("conditioning wrapper should succeed");

        assert_eq!(report.error_level, 0);
        assert_eq!(report.assessment_type.as_deref(), Some("Conditioning"));
        assert_eq!(report.iid, Some(false));
        assert!(!report.test_cases.is_empty());
    }
}
