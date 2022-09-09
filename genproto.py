from re import sub
import subprocess

paths = [
        "cstrike15_gcmessages.proto",
        "cstrike15_usermessages.proto",
        "netmessages.proto",
        "steammessages.proto"
        ]

for path in paths:
    subprocess.call(f"protoc -I=/home/laiho/Documents/programming/rust/demoparse/Protobufs/ --rust_out=/home/laiho/Documents/programming/rust/demoparse/temp/ /home/laiho/Documents/programming/rust/demoparse/Protobufs/{path}")