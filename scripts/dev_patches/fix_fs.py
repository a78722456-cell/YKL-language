
import re

with open("std/fs.yk", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("extern \"C\" {", "extern {")
content = re.sub(r"return\s+Result::Err\((.*?)\);", r"Result::Err(\1)", content)
content = re.sub(r"return\s+Result::Ok\((.*?)\);", r"Result::Ok(\1)", content)

with open("std/fs.yk", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

