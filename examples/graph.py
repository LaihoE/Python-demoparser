import matplotlib.pyplot as plt



names = ["lil lobster", "olavi", "eZz1oS", "-osmanli-", "Cat", "ExΩtiC", "Makezu32"]
granade_dmg = [30, 4, 3, 2, 2, 1, 1]

plt.bar(names,granade_dmg)
plt.title("Number of times that player has hurt a hostage")
plt.show()