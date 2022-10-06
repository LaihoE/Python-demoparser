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


1000 Faceit demos in parallel (12 processes):
```Python
1000/1000 [01:02<00:00, 16.11it/s]
```
1 minute 2 seconds with an average of 16.11 demos per second. The SSD im using is very fast so your numbers might vary greatly.