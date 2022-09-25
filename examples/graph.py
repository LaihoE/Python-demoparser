import matplotlib.pyplot as plt


names = ["osku", "Laiho", "olavi", "-ExΩtiC-", "Makezu32"]
granade_dmg = [2655, 3424, 3435, 6573, 8828]

plt.bar(names,granade_dmg)
plt.title("total he-granade damage last 200 downloaded demos")
plt.show()