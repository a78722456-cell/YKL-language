
with open("std/vec.yk", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("Vec::<T> {", "Vec {")

with open("std/vec.yk", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

