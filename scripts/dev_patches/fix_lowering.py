with open('crates/apl_sema/src/lib.rs', 'r', encoding='utf-8') as f:
    sema = f.read()

# We'll extract analyze_program to lower_program
# This is a bit complex for a simple regex script. 
