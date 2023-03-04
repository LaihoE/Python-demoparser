v = set()

with open("v.txt") as f:

    lines = f.readlines()
    for line in lines:
        line = line.strip("\n")
        v.add(line)

print(v)

i = 202
for x in v:
    print(f'"{x}" => {i},')
    i += 1

