### Game event to SQL
Exporting outputs to SQL is trivial due to pandas integration with sql.

Example uses sqlite (part of python standard library) but pandas supports more or less every flavor of sql. All you need is to pass pandas the connection to the DB.

Running the example creates a sqlite db into this directory and inserts all kills into table "player_deaths"