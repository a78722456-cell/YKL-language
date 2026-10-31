
import os
import subprocess
import re

TESTS_DIR = "tests"
COMPILER_CMD = ["cargo", "run", "--bin", "apl", "--", "run"]

def run_compile_pass():
    passed = 0
    failed = 0
    dir_path = os.path.join(TESTS_DIR, "compile-pass")
    
    if not os.path.exists(dir_path): return 0, 0
    
    for file in os.listdir(dir_path):
        if not file.endswith(".yk"):
            continue
        
        filepath = os.path.join(dir_path, file)
        result = subprocess.run(COMPILER_CMD + [filepath], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(f"[PASS] {filepath}")
            passed += 1
        else:
            print(f"[FAIL] {filepath}")
            print(result.stdout)
            print(result.stderr)
            failed += 1
            
    return passed, failed

def run_compile_fail():
    passed = 0
    failed = 0
    dir_path = os.path.join(TESTS_DIR, "compile-fail")
    
    if not os.path.exists(dir_path): return 0, 0
    
    for file in os.listdir(dir_path):
        if not file.endswith(".yk"):
            continue
            
        filepath = os.path.join(dir_path, file)
        
        expected_errors = []
        with open(filepath, "r", encoding="utf-8") as f:
            for line in f:
                match = re.search(r"//~\s*ERROR:\s*(.*)", line)
                if match:
                    expected_errors.append(match.group(1).strip())
                    
        result = subprocess.run(COMPILER_CMD + [filepath], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(f"[FAIL] {filepath} - Expected compilation failure, but it succeeded!")
            failed += 1
            continue
            
        output = result.stdout + result.stderr
        all_found = True
        for expected in expected_errors:
            if expected not in output:
                print(f"[FAIL] {filepath} - Expected error not found: '{expected}'")
                all_found = False
                break
                
        if all_found:
            print(f"[PASS] {filepath}")
            passed += 1
        else:
            print(f"[FAIL] Output was:\n{output}")
            failed += 1
            
    return passed, failed

if __name__ == "__main__":
    print("Running YKL Test Suite...")
    p1, f1 = run_compile_pass()
    p2, f2 = run_compile_fail()
    print("-" * 30)
    print(f"Total: {p1+p2} passed, {f1+f2} failed.")
    if f1 > 0 or f2 > 0:
        exit(1)

