v = set()

with open("v.txt") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip("\n")
        v.add(line)


d = {41: 70, 39:71, 43:67}

print(v)

i = 202
for x in v:
    z = x.split(" ")

    print(f'"{z[0]}" => {d[int(z[1])]},')

