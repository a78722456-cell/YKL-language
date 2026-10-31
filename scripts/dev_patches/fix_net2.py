
with open("std/net.yk", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("Result::Err(-1);", "Result::Err(-1)")
content = content.replace("Result::Ok(server_fd);", "Result::Ok(server_fd)")
content = content.replace("Result::Ok(client_fd);", "Result::Ok(client_fd)")
content = content.replace("Result::Ok(bytes);", "Result::Ok(bytes)")
content = content.replace("0;", "0")

with open("std/net.yk", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

