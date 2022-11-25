### Examples for extracting per round statistics


These vars are quite odd, they seem like they should update "live" but in reality they update one time per round. 
Probably the easiest way is to just query all ticks and drop duplicates. If performance is critical you might consider querying less ticks 
for example every 100 ticks or similar.

Also small warning NaN appears instead of 0 for these values. Might change behaviour in future.