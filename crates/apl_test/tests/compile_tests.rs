use apl_test::{compile_source, format_diagnostics};
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn run_compile_fail_tests() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // CARGO_MANIFEST_DIR is crates/apl_test, so we go up to the workspace root
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let compile_fail_dir = workspace_root.join("tests").join("compile_fail");
    let snapshots_dir = workspace_root.join("tests").join("snapshots");

    if !snapshots_dir.exists() {
        fs::create_dir_all(&snapshots_dir).unwrap();
    }

    let update_expect = env::var("UPDATE_EXPECT").map(|v| v == "1").unwrap_or(false);

    let mut failed = false;

    for entry in fs::read_dir(compile_fail_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yk") {
            continue;
        }

        let filename = path.file_name().unwrap().to_str().unwrap();
        let source = fs::read_to_string(&path).unwrap();

        let output = compile_source(&source, filename);
        let stderr_output = format_diagnostics(&output.diagnostics, filename, &source);

        let snapshot_path = snapshots_dir.join(path.with_extension("stderr").file_name().unwrap());

        if update_expect {
            fs::write(&snapshot_path, &stderr_output).unwrap();
            println!("Updated snapshot for {}", filename);
        } else {
            if !snapshot_path.exists() {
                println!(
                    "Missing snapshot for {}, run with UPDATE_EXPECT=1",
                    filename
                );
                failed = true;
                continue;
            }
            let expected = fs::read_to_string(&snapshot_path).unwrap();

            // Normalize line endings for Windows/Linux comparison
            let expected_norm = expected.replace("\r\n", "\n");
            let actual_norm = stderr_output.replace("\r\n", "\n");

            if expected_norm != actual_norm {
                println!(
                    "Mismatch in {}:\nExpected:\n{}\nActual:\n{}",
                    filename, expected_norm, actual_norm
                );
                failed = true;
            }
        }
    }

    assert!(!failed, "Some snapshot tests failed");
}
