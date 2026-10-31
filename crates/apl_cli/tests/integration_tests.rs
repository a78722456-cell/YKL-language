use std::fs;
use std::process::Command;

#[test]
fn test_compile_pass() {
    let dir = "../../tests/compile-pass";
    if !std::path::Path::new(dir).exists() { return; }
    
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap_or_default() == "yk" {
            let status = Command::new("cargo")
                .args(["run", "-p", "apl_cli", "--", "run", path.to_str().unwrap()])
                .status()
                .expect("Failed to run compiler");
            
            assert!(status.success(), "Test failed: {}", path.display());
        }
    }
}

#[test]
fn test_compile_fail() {
    let dir = "../../tests/compile-fail";
    if !std::path::Path::new(dir).exists() { return; }
    
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap_or_default() == "yk" {
            let status = Command::new("cargo")
                .args(["run", "-p", "apl_cli", "--", "run", "--dump-borrowck", path.to_str().unwrap()])
                .status()
                .expect("Failed to run compiler");
            
            assert!(!status.success(), "Test should have failed, but passed: {}", path.display());
        }
    }
}
