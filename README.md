Benchmarks for an upcoming blog post on comparing byte- and bit-level parsing with Nom.

Nom can parse strings one byte at a time, e.g. "11101010" as eight chars.
But it can also parse bytes one bit at a time, e.g. the number `0b11101010` as eight bits.

I'm sure the bitwise parsing is faster, but I'm not sure how much faster.
