
import re

with open("std/net.yk", "r", encoding="utf-8") as f:
    content = f.read()

content = re.sub(r"return\s+Result::Err\((.*?)\);", r"Result::Err(\1)", content)
content = re.sub(r"return\s+Result::Ok\((.*?)\);", r"Result::Ok(\1)", content)

with open("std/net.yk", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

