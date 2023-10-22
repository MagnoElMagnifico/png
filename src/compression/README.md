<!--
TODO:
<details>
<summary><b>Table of Contents</b> (click to open)</summary>

1. [Helpful Links:](#helpful-links)
1. [Sublime Text Settings:](#sublime-text-settings)
1. [Packages to install](#packages-to-install)

</details>

...
<a name="paragraph2"></a>
<div id='id-section2'/>
-->


> "You crazy, don't reinvent the wheel! Use a [library](https://crates.io/crates/flate2)!"

**Sources**: [PNG Wikipedia], [zlib.net], [thuc.space], [Deflate spec] and [libPNG Chapter 9].

[PNG Wikipedia]:    https://en.wikipedia.org/wiki/PNG#Compression
[Huffman codes]:    https://en.wikipedia.org/wiki/Huffman_coding
[LZ77]:             https://en.wikipedia.org/wiki/LZ77_and_LZ78
[zlib.net]:         https://zlib.net/feldspar.html
[Deflate spec]:     https://www.rfc-editor.org/rfc/rfc1951
[libPNG Chapter 9]: http://www.libpng.org/pub/png/book/chapter09.html
[thuc.space]:       https://thuc.space/posts/deflate/

[codersnotes.com]: http://www.codersnotes.com/notes/elegance-of-deflate/
[code]: https://github.com/Frommi/miniz_oxide
[debugger]: https://github.com/madler/infgen/


# `LZ77`

`LZ77` works finding sequences of data that are repeated and replacing them
with a pointer, consisting in a distance and a length:

- The distance holds a reference to the first appearance of the sequence,
- and the length stores the length of the sequence.

As an example, the text `this is` has the sequence `is` repeated, thus it can
be replaced with a pointer: `this <3,2>`.

```
           Length: 2
           ───
┌─┬─┬─┬─┬─┬─┬─┐
│t│h│i│s│ │i│s│
└─┴─┴─┴─┴─┴─┴─┘
     ▲     │
     │     │
     └─────┘
     Distance: back 3
```

The term _sliding window_ is used, all it means is that at any given point,
there is a record of what symbols went before. A 32K window is often used.

Consider this other example:

```
 vvvvv
Blah blah blah blah blah!
      ^^^^^
```

Therefore, the first part of the data (`Blah blah b`) can be represented as `Blah b<5,5>`.

However, this can be further compressed: compare the character that follows
each of them. In both cases, it's `l` -- so we can make the length 6, and not
just five. But if we continue checking, we find the next characters are still
identical -- even if the so-called _previous_ string is overlapping the string
we're trying to represent in the compressed data!

It's true that when we're decompressing, and read the length, distance pair
that describes this relationship, we don't know what all those next characters
will be yet -- but if we put in place the ones that we know, we will know more,
which will allow us to put down more... Or, knowing that any
length-and-distance pair where `length > distance` is going to be repeating
(distance) characters again and again, we can set up the decompressor to do
just that.

It turns out our highly compressible data can be compressed down to just this:

```
Blah b<5,18>!
```


# Huffman coding

The Huffman coding replaces symbols from original document (alphabet) to mapping codes.
The mapping codes can be different sizes, from 1 bit until unlimited bits.

These mappings prioritize smaller codes for more frequent alphabet elements.

Also, the codes in the Huffman coding algorithm is prefix code. It means no any
code is a prefix of another code. It avoid to confusing between encode/decode
symbols and series of codes.

```
        /\              Symbol    Code
       0  1             ------    ----
      /    \                A      00
     /\     B               B       1
    0  1                    C     011
   /    \                   D     010
  A     /\
       0  1
      /    \
     D      C
```

1. Analyse symbols and their weight (number of occurs) from an original document.
2. Build a binary tree. Middle nodes contain temporary code, leaves contain
   mapping between symbols and their mapping codes. Keeps highest weight symbol
   on one side of the binary tree, lower weight symbol try to add to another
   side of the binary tree.
3. Collect leaves and their code. Replace symbols with codes.

Example in [thuc.space Huffman].

Other approach to step 2 is using a greedy algorithm, explained in [Abdul Bari video]:

1. Take the two minimum weighted symbols and let the weight of the new node be
   the sum of their frequencies.
2. Repeat this process now using the new node's weight as a frequency.

Now, to encode the values, assign to the left branch a `0` (smaller
frequencies) and a `1` to the right branch (higher frequencies) and collect the
bits from the root to the leaves.

Note that, to decode the original document, we also have to store this tree / table.

[thuc.space Huffman]: https://thuc.space/posts/huffman_coding_algorithm/
[Abdul Bari video]: https://youtu.be/co4_ahEDCho?t=525


## Huffman codes in DEFLATE

In the DEFLATE format, the Huffman codes for the various alphabets must not
exceed certain maximum code lengths. This constraint complicates the algorithm
for computing code lengths from symbol frequencies.

<!-- TODO: See [1] Chapter 5 https://www.rfc-editor.org/rfc/rfc1951 -->

Also, the Huffman codes used for each alphabet in the DEFLATE format have two
additional rules:

- Shorter codes lexicographically procede longer codes.

  I.e.: elements that have shorter codes are placed to the left of those with
  longer codes.

- All codes of a given bit length have to lexicographically consecutive values,
  in the same order as the symbols they represent.

  I.e.: Among elements with codes of the same length, those that come first in
  the element set are placed to the left.

Let's see an example. Assuming that the order of the alphabet is `ABCD`:

```
Symbol  Code
------  ----
A       10
B       0
C       110
D       111
```

Therefore, `0` precedes `10` which precedes `11x`. `110` and `111` are
lexicographically consecutive.

Given this rule, we can define the Huffman code for an alphabet just by giving
the bit lengths of the codes for each symbol of the alphabet in order. In this
example, `(2, 1, 3, 3)`.

The following algorithm generates the codes as integers, intended to be read
most- to least-significant bit. The code lengths are initially in `tree[I].Len`
and the codes are produced in `tree[I].Code`.

1. Count the number of codes for each code length. Let `lb_count[N]` be the
   number of codes of length `N`.

2. Find the numerical value of the smallest code for each code length:

   ```c
   code = 0;
   bl_count[0] = 0;
   for (bits = 1; bits <= MAX_BITS; bits++) {
       code = (code + bl_count[bits-1]) << 1;
       next_code[bits] = code;
   }
   ```

3. Assign numerical values to all codes, using consecutive values for all codes
   of the same length with the base values determined at step 2. Codes that are
   never used (which have a bit length of 0) must not be assign a value.

   ```c
   for (n = 0; n <= max_code; n++) {
       len = tree[n].Len;
       if (len != 0) {
           tree[n].Code = next_code[len];
           next_code[len]++;
       }
   }
   ```

## Example

Consider the alphabet `ABCDEFGH` with bit lengths `(3, 3, 3, 3, 3, 2, 4, 4)`.

Step 1:

```
N            length of the code
lb_count[N]  number of codes with length N

N      bl_count[N]
-      -----------
2      1
3      5
4      2
```

Step 2:

```
N      next_code[N]
-      ------------
1      0
2      0
3      2     =   10
4      14    = 1110
```

Step 3:

```
Symbol Length   Code
------ ------   ----
A       3        010
B       3        011
C       3        100
D       3        101
E       3        110
F       2         00
G       4       1110
H       4       1111
```


# Compression

PNG uses DEFLATE, which is a non-patented lossless data compression algorithm.
A compressed data set consists of a series of blocks using a combination of
[LZ77] and [Huffman codes].

The Huffman trees for each block are independent of those for previous or
subsequent blocks; and LZ77 may use a reference to a duplicated string occurring
in a previous block, up to 32K input bytes before.

The representation used limits distances to 32K bytes and lengths to 258 bytes
for the LZ77 pointers, but does not limit the size of a block (except for
uncompressible blocks, limited to 65 535 bytes).

<!--
## Binary format

- Data elements are packed into bytes starting with the least-significant bit
  of the byte.
- Data elements other than the Huffman codes are packed starting with the
  least-significant bit of the data element.
- Huffman codes are packed starting with the most-significant bit of the code.

In other words, if one were to print out the compressed data as a sequence of
bytes, starting with the first byte at the *right* margin and proceeding to the
*left*, with the most- significant bit of each byte on the left as usual, one
would be able to parse the result from right to left, with fixed-width elements
in the correct MSB-to-LSB order and Huffman codes in bit-reversed order (i.e.,
with the first bit of the code in the relative LSB position).
--->


## Block format

Each block consists in the following:

- A **header** (3 bits):
    - First bit: `BFINAL`. It is only set if this is the last block of the data set.
    - Next 2 bits: `BTYPE`. It specifies how the data are compressed:

    ```
    00 - no compression
    01 - compressed with fixed Huffman codes
    10 - compressed with dynamic Huffman codes
    11 - reserved (error)
    ```
    The only difference between the two compressed cases is how the Huffman codes
    are defined.

- A pair of **Huffman code trees** in compact form just before the compressed
  data.

- A **compressed data** part. This part consists in a series of elements of two
  types:

   - **Literal** bytes of strings that have not been detected as duplicated:
     `0..255`

   - **Pointers** to duplicated strings (represented as `<length, backward
     distance>`): length `3..258`, distance `1..32 768`.

<!--
These parts are represented using a Huffman code:

- huffman trees compressed using Huffman encoding   <--- How???
- One tree for literals and lengths
- and a separated code tree for distances
-->


### Length and distance codes

Literals and lengths can be merged into the same `0..285`:

- `0..255`: literals
- `256`: end of block
- `257..285`: length codes in conjunction with extra bits as follows:

```
     Extra                   Extra                    Extra
Code Bits Length(s)     Code Bits Lengths        Code Bits Length(s)
---- ---- ------        ---- ---- -------        ---- ---- -------
 257   0     3           267   1   15,16          277   4   67-82
 258   0     4           268   1   17,18          278   4   83-98
 259   0     5           269   2   19-22          279   4   99-114
 260   0     6           270   2   23-26          280   4  115-130
 261   0     7           271   2   27-30          281   5  131-162
 262   0     8           272   2   31-34          282   5  163-194
 263   0     9           273   3   35-42          283   5  195-226
 264   0    10           274   3   43-50          284   5  227-257
 265   1  11,12          275   3   51-58          285   0    258
 266   1  13,14          276   3   59-66
 ```

A distance code is required when the literals/length are in `257..285`:

```
     Extra                 Extra                   Extra
Code Bits Dist        Code Bits   Dist         Code Bits Distance
---- ---- ----        ---- ----  ------        ---- ---- --------
  0   0    1           10   4     33-48        20    9   1025-1536
  1   0    2           11   4     49-64        21    9   1537-2048
  2   0    3           12   5     65-96        22   10   2049-3072
  3   0    4           13   5     97-128       23   10   3073-4096
  4   1   5,6          14   6    129-192       24   11   4097-6144
  5   1   7,8          15   6    193-256       25   11   6145-8192
  6   2   9-12         16   7    257-384       26   12  8193-12288
  7   2  13-16         17   7    385-512       27   12 12289-16384
  8   3  17-24         18   8    513-768       28   13 16385-24576
  9   3  25-32         19   8   769-1024       29   13 24577-32768
```

The extra bits should be interpreted as a machine integer stored with the
most-significant bit first, e.g., bits 1110 represent the value 14.

Some examples of lengths:

```
Code   Code without extra bits  Code with extra bits  Length
-----  -----------------------  --------------------  ------
 265    100001001                100001001 0           11
 265    100001001                100001001 1           12
 274    100010010                100010010 000         43
 274    100010010                100010010 001         44
```

And some examples of distances:

```
Code   Code without extra bits  Code with extra bits  Distance
-----  -----------------------  --------------------  --------
 6      110                      110 00                 9
 6      110                      110 01                10
 6      110                      110 10                11
 6      110                      110 11                12
```


### Non-compressed blocks (`BTYPE=00`)

```
+------------+-------------+
| BFINAL     | BTYPE = 00  |  Header
+------------+-------------+ ---------
| LEN 2bytes | NLEN 2bytes |  Data
+------------+-------------+
|                          |
|       Literal data       |
|                          |
+--------------------------+
```

`LEN` is the number of data bytes in the block. `NLEN` is the one's complement
of `LEN`.
<!-- Why?? -->


### Compression with fixed Huffman codes (`BTYPE=01`)

```
+------------+-------------+
| BFINAL     | BTYPE = 01  |  Header
+------------+-------------+ ---------
|                          |  Data
|      Compressed data     |
|                          |
+--------------------------+
```

The Huffman codes for the two alphabets are fixed and not represented
explicitly in the data.

```
Lit Value    Bits  Codes
---------    ----  -----
  0 - 143     8      00110000 to  10111111
144 - 255     9     110010000 to 111111111
256 - 279     7       0000000 to   0010111
280 - 287     8      11000000 to  11000111
```

The code lengths should be sufficient to generate the actual codes, but they
were added for extra clarity.

Distance codes `0-31` are represented by (fixed-length) 5-bit codes, with
possible additional bits as shown in the table shown above.

Also note that literal/length values from `286-287` and distance codes `30-31`
will never be used, but participate in the code construction.


## Compression with dynamic Huffman codes (`BTYPE=10`)

```
+------------+-------------+
| BFINAL     | BTYPE = 01  |  Header
+------------+-------------+ ---------
|    Huffman code trees    |  Data
| literal/length, distance |
+--------------------------+
|                          |
|      Compressed data     |
|                          |
+--------------------------+
```

The Huffman codes appear immediately after the header block, first the
literals/lengths and then the distances.

For even greater compactness, the code length sequences are algo encoded using
a Huffman code:

```
0 - 15   Represent code lengths of 0 - 15
    16   Copy the previous code length 3 - 6 times.
         The next 2 bits indicate repeat length (0 = 3, ... , 3 = 6)
             Example:  Codes 8, 16 (+2 bits 11),
                       16 (+2 bits 10) will expand to
                       12 code lengths of 8 (1 + 6 + 5)
    17   Repeat a code length of 0 for 3 - 10 times.
       (3 bits of length)
    18   Repeat a code length of 0 for 11 - 138 times
       (7 bits of length)
```

# Decoding algorithm

```
do
   read block_header from input_stream

   if stored with no compression
      skip any remaining bits in current partially processed byte
      read LEN and NLEN
      copy LEN bytes of data to output

   else
      if compressed with dynamic Huffman codes
         read representation of code trees

      loop (until end_of_block code recognized)
         decode literal/length value from input stream

         if value < 256
            copy value (literal byte) to output stream

         else
            if value = end_of_block (256)
               break

            else (value = 257..285)
               decode distance from input_stream
               move backwards distance bytes in the output stream
               copy length bytes from this position to the output stream
      end loop
while not last_block
```
