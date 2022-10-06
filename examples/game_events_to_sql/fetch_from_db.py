import sqlite3


con = sqlite3.connect("testing_database.sqlite")
cur = con.cursor()

# Fetch all awp kills
results = cur.execute("SELECT * FROM player_death WHERE weapon='awp'").fetchall()
print(results)


# Elegant way to get query to df
# df = pd.read_sql_query("SELECT * player_death", con)