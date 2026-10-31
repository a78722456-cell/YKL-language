
with open("std/result.yk", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("""        Result::Ok(val) => {
            return val;
        },""", """        Result::Ok(val) => {
            val
        },""")
content = content.replace("""        Result::Err(e) => {
            return default_val;
        },""", """        Result::Err(e) => {
            default_val
        },""")

content = content.replace("""        Result::Ok(val) => {
            return true;
        },""", """        Result::Ok(val) => {
            true
        },""")
content = content.replace("""        Result::Err(e) => {
            return false;
        },""", """        Result::Err(e) => {
            false
        },""")
content = content.replace("""        Result::Ok(val) => {
            return false;
        },""", """        Result::Ok(val) => {
            false
        },""")
content = content.replace("""        Result::Err(e) => {
            return true;
        },""", """        Result::Err(e) => {
            true
        },""")

with open("std/result.yk", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

