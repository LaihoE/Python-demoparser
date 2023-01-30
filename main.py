import numpy as np
import random

a = np.load("data.npy")

print(a.shape[0])

x = [random.randint(0, a.shape[0] - 500) for _ in range(100000)]

out = []
for s in x:
    z = a[s:s+256]
    if len(np.unique(z)) > 50:
        out.append(z.reshape(1, -1))

a = np.concatenate(out, axis=0)
print(a.shape)


np.save("emil", a)