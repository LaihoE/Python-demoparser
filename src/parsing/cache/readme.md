# Cache


The cache can be thought of as a data frame with 4 columns like so:

| byte   | tick  | entity id | prop idx |
| ------ | ----- | --------- | -------- |
| 123456 | 1234  | 0         | 20       |
| 123456 | 1234  | 1         | 21       |
| 288227 | 28222 | 4         | 20       |

Having one row per updated value. While this simplicity would be great, unfortunately the file would become too big and too slow. We have to optimize. Time spent here can easily become bigger than time spent parsing the entire demo.

## Constraints
The 3 main things that we need to focus on is small file size, fast read and somewhat fast write. We only write one time so some extra time can be spent here.

Especially important is to be able to quickly find the correct rows for rare props that are only set a couple of times during the demo. We don't want to uncompress the entire cache just to be able to find where players ranks are stored. This leads to Optimization 1:

## Optimization 1 - Store each prop seperately
While this makes the file bigger, this is almost a must for fast performance. Currently the cache is just a ZIP archive with one file per prop. I'm not completely happy with ZIP due to the central header being somewhat expensive to parse when you have many files. HDF5 is the other obvious alternative, but the dependencies seem quite heavy and im not sure about how fast its "central header" is to parse. Please share if you have other suggestions.


## Optimization 2 - Byte and tick to id


Byte and tick go hand in hand. Meaning that a byte always maps to the same tick and vice versa.

Generate ids for each byte tick pair. There is currently no magic here. Just count from 0 up.
The above table would now look like this:

| id  | entity id | prop idx |
| --- | --------- | -------- |
| 0   | 0         | 20       |
| 0   | 1         | 21       |
| 1   | 4         | 20       |

and then we store the id -> (byte, tick) mapping seperately (in some key value form)

Ids could potentially be created in a smarter way, for example based on how common it is, but this might quickly become expensive. The more common the value the smaller the id. Not sure how much this helps with the compression.

## Optimization 3 - One row per id prop pair

Another problem with the above is that we have quite a lot of rows with the same id and prop idx. For example if all 10 players are moving then we will get something like this.


| id  | entity id | prop idx |
| --- | --------- | -------- |
| 55  | 2         | 20       |
| 55  | 3         | 20       |
| 55  | 4         | 20       |
| 55  | 5         | 20       |
| 55  | 6         | 20       |
| 55  | 7         | 20       |
| 55  | 8         | 20       |
| 55  | 9         | 20       |
| 55  | 10        | 20       |
| 55  | 11        | 20       |

We can see quite a lot of redundancy here. While this is something a compression algorithm might do quite well on, it will cause quite a lot of memory being used -> slower.
The solution for now is to use bitflags to signal which entity was updated. Like this (here we use u16):


| id  | entity ids bitflags | prop idx |
| --- | ------------------- | -------- |
| 55  | 0000111111111110    | 20       |

Here we set bits 2-11 (from right to left, first being 1). I think a similar idea can be found in the demo itself in some places. If I remeber correctly then "spotted by" prop works the same way.