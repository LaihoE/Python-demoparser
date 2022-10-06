## Example of finding all aces from a list of demos

Output is something like this:

<pre>round attacker_name    attacker_id  total_kills       file
2      Flusha  76561197991348083            5         demo1.dem
15     Flusha  76561197991348083            5         demo2.dem
27     Flusha  76561197991348083            5         demo3.dem
</pre>


## Performance: 

Specs: 
Ryzen 9 5900x
Samsung 980 pro nvme SSD


952 Faceit demos in parallel (12 processes):
```Python
952/952 [00:27<00:00, 35.16it/s]
```
took: 27s average 35 demos / second. The SSD im using is very fast so your numbers might vary greatly.