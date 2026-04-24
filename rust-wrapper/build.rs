use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("rust-wrapper should live under the repository root");
    let cpp_dir = env::var_os("SP80090B_CPP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("vendor/SP800-90B_EntropyAssessment/cpp"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("missing OUT_DIR"));
    let bin_dir = out_dir.join("bin");

    assert!(
        cpp_dir.is_dir(),
        "missing upstream C++ source directory at {}",
        cpp_dir.display()
    );

    fs::create_dir_all(&bin_dir).expect("failed to create binary output directory");
    emit_rerun_instructions(&cpp_dir);

    let compiler = resolve_compiler();
    let jsoncpp_flags = probe_pkg_config(&["--cflags", "--libs", "jsoncpp"]);
    let extra_cxxflags = split_env_flags("SP80090B_CXXFLAGS");
    let extra_ldflags = split_env_flags("SP80090B_LDFLAGS");

    let tools = [
        Tool {
            source: "iid_main.cpp",
            output: "ea_iid",
            libs: &[
                "-lbz2",
                "-lpthread",
                "-ldivsufsort",
                "-ldivsufsort64",
                "-ljsoncpp",
                "-lcrypto",
            ],
        },
        Tool {
            source: "non_iid_main.cpp",
            output: "ea_non_iid",
            libs: &[
                "-lbz2",
                "-lpthread",
                "-ldivsufsort",
                "-ldivsufsort64",
                "-ljsoncpp",
                "-lcrypto",
            ],
        },
        Tool {
            source: "restart_main.cpp",
            output: "ea_restart",
            libs: &[
                "-lbz2",
                "-lpthread",
                "-ldivsufsort",
                "-ldivsufsort64",
                "-ljsoncpp",
                "-lcrypto",
            ],
        },
        Tool {
            source: "conditioning_main.cpp",
            output: "ea_conditioning",
            libs: &[
                "-lbz2",
                "-lpthread",
                "-ldivsufsort",
                "-ldivsufsort64",
                "-ljsoncpp",
                "-lmpfr",
                "-lgmp",
                "-lcrypto",
            ],
        },
    ];

    for tool in tools {
        compile_tool(
            &compiler,
            &cpp_dir,
            &bin_dir,
            &jsoncpp_flags,
            &extra_cxxflags,
            &extra_ldflags,
            tool,
        );
    }
}

struct Tool<'a> {
    source: &'a str,
    output: &'a str,
    libs: &'a [&'a str],
}

fn compile_tool(
    compiler: &str,
    cpp_dir: &Path,
    bin_dir: &Path,
    jsoncpp_flags: &[OsString],
    extra_cxxflags: &[OsString],
    extra_ldflags: &[OsString],
    tool: Tool<'_>,
) {
    let source = cpp_dir.join(tool.source);
    let output = bin_dir.join(tool.output);
    let mut command = Command::new(compiler);
    let clang_like = is_clang_like(compiler);

    command.current_dir(cpp_dir);
    command.arg("-std=c++11");
    command.arg("-O2");
    command.arg("-ffloat-store");
    add_openmp_flags(&mut command, clang_like);

    add_if_exists(&mut command, "-I/usr/include/jsoncpp");
    add_if_exists(&mut command, "-I/opt/homebrew/include");
    add_if_exists(&mut command, "-L/opt/homebrew/lib");
    add_if_exists(&mut command, "-I/usr/local/include");
    add_if_exists(&mut command, "-L/usr/local/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/jsoncpp/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/jsoncpp/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/libdivsufsort/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/libdivsufsort/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/openssl@3/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/openssl@3/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/bzip2/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/bzip2/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/gmp/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/gmp/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/mpfr/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/mpfr/lib");
    add_if_exists(&mut command, "-I/opt/homebrew/opt/libomp/include");
    add_if_exists(&mut command, "-L/opt/homebrew/opt/libomp/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/jsoncpp/include");
    add_if_exists(&mut command, "-L/usr/local/opt/jsoncpp/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/libdivsufsort/include");
    add_if_exists(&mut command, "-L/usr/local/opt/libdivsufsort/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/openssl@3/include");
    add_if_exists(&mut command, "-L/usr/local/opt/openssl@3/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/bzip2/include");
    add_if_exists(&mut command, "-L/usr/local/opt/bzip2/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/gmp/include");
    add_if_exists(&mut command, "-L/usr/local/opt/gmp/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/mpfr/include");
    add_if_exists(&mut command, "-L/usr/local/opt/mpfr/lib");
    add_if_exists(&mut command, "-I/usr/local/opt/libomp/include");
    add_if_exists(&mut command, "-L/usr/local/opt/libomp/lib");

    for flag in jsoncpp_flags {
        command.arg(flag);
    }
    for flag in extra_cxxflags {
        command.arg(flag);
    }

    command.arg(source);
    command.arg("-o");
    command.arg(output);

    for lib in tool.libs {
        command.arg(lib);
    }
    if clang_like {
        command.arg("-lomp");
    }
    for flag in extra_ldflags {
        command.arg(flag);
    }

    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to invoke {compiler}: {err}"));
    if !status.success() {
        panic!("failed to compile {} with status {}", tool.output, status);
    }
}

fn emit_rerun_instructions(cpp_dir: &Path) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let mut stack = vec![cpp_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
        for entry in entries {
            let entry = entry.expect("failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}

fn probe_pkg_config(args: &[&str]) -> Vec<OsString> {
    let Ok(output) = Command::new("pkg-config").args(args).output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(OsString::from)
        .collect()
}

fn split_env_flags(name: &str) -> Vec<OsString> {
    env::var_os(name)
        .map(|value| {
            value
                .to_string_lossy()
                .split_whitespace()
                .map(OsString::from)
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_compiler() -> String {
    if let Some(cxx) = env::var_os("CXX") {
        return cxx.to_string_lossy().into_owned();
    }

    if Path::new("/opt/homebrew/opt/libomp/lib/libomp.dylib").exists() && compiler_exists("clang++")
    {
        return "clang++".to_string();
    }

    let candidates = [
        "g++",
        "/opt/homebrew/bin/g++-15",
        "/opt/homebrew/bin/g++-14",
        "/opt/homebrew/bin/g++-13",
        "/usr/local/bin/g++-15",
        "/usr/local/bin/g++-14",
        "/usr/local/bin/g++-13",
        "c++",
    ];

    for candidate in candidates {
        if compiler_exists(candidate) {
            return candidate.to_string();
        }
    }

    "c++".to_string()
}

fn compiler_exists(candidate: &str) -> bool {
    if candidate.contains('/') {
        return Path::new(candidate).exists();
    }

    Command::new("which")
        .arg(candidate)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn add_if_exists(command: &mut Command, flag: &str) {
    if let Some(path) = flag.strip_prefix("-I").or_else(|| flag.strip_prefix("-L")) {
        if Path::new(path).exists() {
            command.arg(flag);
        }
    }
}

fn is_clang_like(compiler: &str) -> bool {
    Path::new(compiler)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(compiler)
        .contains("clang")
}

fn add_openmp_flags(command: &mut Command, clang_like: bool) {
    if clang_like {
        command.arg("-Xpreprocessor");
        command.arg("-fopenmp");
    } else {
        command.arg("-fopenmp");
    }
}
