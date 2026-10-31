import re

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# I will just replace the exact block
block_to_replace = '''                _ => {}
            }
            }
        }
    }
}'''

replacement = '''                _ => {}
            }
        }
    }
}'''

text = text.replace(block_to_replace, replacement)

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
