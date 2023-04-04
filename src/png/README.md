> Note: In the code there are comments that summarize the [official
> specification] and the [W3 Docs].


[official specification]: http://libpng.org/pub/png/spec/1.2/PNG-Contents.html
[W3 Docs]: https://www.w3.org/TR/png-3/


# PNG Basics

The PNG format consists in a **signature** and a series of **chunks** (PNG's
read and write operations are defined in the `png` module). This filetype is a
Resource Interchange File Format (RIFF).

Each chunk has the following structure (module `chunks`):

- Length of the data section (4 bytes)
- Chunk type code (4 bytes)
- Chunk data section
- Cyclic redundancy check (4 bytes): Error-detecting code (calculated in module
  `crc`)

Main chunks:

- `IHDR`: starts the PNG file and contains basic information such as the size,
  bit depth, compression methods, etc.

- `IDAT`: **TODO**

- `PLTE`: **TODO**

- `IEND`: empty chunk marking the end of the file.

Optional chunks:

- Transparency: `tRNS`
- Color space: `gAMA`, `cHRM`, `sRGB`, `iCCP`
- Text: `iTXt`, `tEXt`, `zTXt`
- Miscellaneous: `bKGD`, `pHYs`, `sBIT`, `sPLT`, `hIST`, `tIME`

```
   Critical chunks (must appear in this order, except PLTE
                    is optional):

           Name  Multiple  Ordering constraints
                   OK?

           IHDR    No      Must be first
           PLTE    No      Before IDAT
           IDAT    Yes     Multiple IDATs must be consecutive
           IEND    No      Must be last

   Ancillary chunks (need not appear in this order):

           Name  Multiple  Ordering constraints
                   OK?

           cHRM    No      Before PLTE and IDAT
           gAMA    No      Before PLTE and IDAT
           iCCP    No      Before PLTE and IDAT
           sBIT    No      Before PLTE and IDAT
           sRGB    No      Before PLTE and IDAT
           bKGD    No      After PLTE; before IDAT
           hIST    No      After PLTE; before IDAT
           tRNS    No      After PLTE; before IDAT
           pHYs    No      Before IDAT
           sPLT    Yes     Before IDAT
           tIME    No      None
           iTXt    Yes     None
           tEXt    Yes     None
           zTXt    Yes     None

Standard keywords for text chunks:

   Title            Short (one line) title or caption for image
   Author           Name of image's creator
   Description      Description of image (possibly long)
   Copyright        Copyright notice
   Creation Time    Time of original image creation
   Software         Software used to create the image
   Disclaimer       Legal disclaimer
   Warning          Warning of nature of content
   Source           Device used to create the image
   Comment          Miscellaneous comment; conversion from GIF comment
```


# Image Layout

A PNG image is a rectangular pixel array, appearing left-to-right within each
_scanline_, and these appearing top-to-bottom.

However, the data may be transmitted in a different order, see [PNG Interlaced].

Pixels are always packed into these scanlines with no wasted bits between
pixels. Pixels smaller than a byte never cross byte boundaries; they are packed
into bytes with the leftmost pixel in the high-order bits of a byte, the
rightmost in the low-order bits.

The size of each pixel is determined by the _bit depth_, which is the number of
bits per sample in the image data.

Pixel types:

- _Indexed color_: single sample that is an index into a color palette.
  Therefore, the _bit depth_ represents the maximum of palette entries, but not
  the color precision.

- _Greyscale_: single sample representing luminance, where 0 is black and the
  largest value for _bit depth_ is white.

- _True color_ (RGB): three samples, where 0 is black and _bit depth_ max is
  Red, Green or Blue. The bit depth specifies the size of each sample, not the
  total pixel size.

Sample values are not necessarily proportional to light intensity; the `gAMA`
chunk specifies the relationship between sample values and display output
intensity.


## Alpha


## Interlaced

[PNG Interlacing Wikipedia]: https://en.wikipedia.org/wiki/PNG#Interlacing
[Adam7]: https://en.wikipedia.org/wiki/Adam7_algorithm
[PNG Interlaced]: http://libpng.org/pub/png/spec/1.2/PNG-DataRep.html#DR.Interlaced-data-order

## Gamma
# Text Strings

# Filter algorithms

Filtering algorithms are applied to IDAT chunks before compression to prepare
the data in order to optimize the resultant size.

The PNG filter method 0, described in the IHDR chunk, is a set of the following
5 filter types.

```
0   None
1   Sub
2   Up
3   Average
4   Paeth
```

For the moment (PNG 1.2) this is the only filter method available.

The data can be encoded with all these algorithms in a scanlines-by-scanline
basis, thus the data send to compress is preceded by a filter-type byte that
specifies the filter algorithm.

![](https://www.w3.org/TR/2003/REC-PNG-20031110/figures/fig49.svg)
Source: [w3](https://www.w3.org/TR/2003/REC-PNG-20031110/#4Concepts.EncodingFiltering)

These are applied to the bytes that conform each scanline, not pixels. If the
image includes an alpha channel, it is filtered in the same way.

When the image is interlaced, each pass is treated as an independent image (**TODO**)

<!--
Interlacing is also a bit of a wrench in the works. For the purposes of
filtering, each interlace pass is treated as a separate image with its own
width and height. For example, in a 256 × 256 interlaced image, the passes
would be treated as seven smaller images with dimensions 32 × 32, 32 × 32, 64 ×
32, 64 × 64, 128 × 64, 128 × 128, and 256 × 128, respectively.[69] This avoids
the nasty problem of how to define corresponding bytes between rows of
different widths.
-->

To decode some filters, you may need to use some of the previous decoded
values, thus the scanline should be stored, since the next scanline might use a
filter that refers to it.

- `None()`: scanline unmodified, only necessary to insert a filter-type byte before the data.
- `Sub()`: transmits the difference between the left pixel and the current one.
- `Up()`: transmits the difference between the top pixel and the current one.
- `Average()`: mix of the previous two, takes the average of the left and top pixel.
- `Paeth()`: applies a simple linear function to the neighbouring pixels.


## How to choose a filtering method

The first rule is that filters are rarely useful on palette images, so don't
even bother with them.

One has considerable freedom in choosing how to order entries in the palette
itself, so it is possible that a particular method of ordering would actually
result in image data that benefits significantly from filtering (not proven).

Filters are also rarely useful on low-bit-depth (grayscale) images in general.

For grayscale and truecolor images (8 or 16 bits per sample), the standard is
_minimum sum of absolute differences_.

The filtered bytes are treated as signed values: any value over 127 is treated
as negative.

```
128 => -128
255 => -1
```

The absolute value of each is then summed, and the filter type that produces
the smallest sum is chosen. This approach effectively gives preference to
sequences that are close to zero and therefore is biased against filter type
None.

A different heuristic (still experimental) might be to favor the most recently
used filter even if its absolute sum of differences is slightly larger than
that of other filters, in order to produce a more homogeneous data stream for
the compressor.

**Source**: [libPNG Chapter 9]


# Compression

PNG uses DEFLATE, which is a non-patented lossless data compression algorithm,
involving a combination of [LZ77] and [Huffman coding].

- **Sources**: [PNG Wikipedia], [zlib.net], [thuc.space], [Deflate spec] and [libPNG Chapter 9].
- **More info**: [Huffman coding] and [LZ77].

> You crazy, don't reinvent the wheel! Use a [library](https://crates.io/crates/flate2)

[PNG Wikipedia]:    https://en.wikipedia.org/wiki/PNG#Compression
[Huffman coding]:   https://en.wikipedia.org/wiki/Huffman_coding
[LZ77]:             https://en.wikipedia.org/wiki/LZ77_and_LZ78
[zlib.net]:         https://zlib.net/feldspar.html
[Deflate spec]:     https://www.rfc-editor.org/rfc/rfc1951
[libPNG Chapter 9]: http://www.libpng.org/pub/png/book/chapter09.html
[thuc.space]:       https://thuc.space/posts/deflate/


## `LZ77`

`LZ77` works finding sequences of data that are repeated and replacing them
with a pointer, consisting in a distance and a length:

- The distance holds a reference to the first appearance of the sequence,
- and the length stores the length of the sequence.

As an example, the text `this is` has the sequence `is` repeated, thus it can
be replaced with a pointer: `this (3,2)`.

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

Therefore, the first part of the data (`Blah blah b`) can be represented as `Blah b(5,5)`.

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
Blah b(5,18)!
```


## Huffman coding

The Huffman coding replaces symbols from original document (alphabet) to mapping codes.
The mapping codes can be different sizes, from 1 bit until unlimited bits.

These mappings prioritize smaller codes for more frequent alphabet elements.

Also, the codes in the Huffman coding algorithm is prefix code. It means no any
code is a prefix of another code. It avoid to confusing between encode/decode
symbols and series of codes.

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

<!--
TODO: Additional Deflate rules

However, there is also the question: how do you pass the tree along with the
encoded data? It turns out that there is a fairly simple way, if you modify
slightly the algorithm used to generate the tree.

In the classic Huffman algorithm, a single set of elements and weights could
generate multiple trees. In the variation used by the Deflate standard, there
are two additional rules: elements that have shorter codes are placed to the
left of those with longer codes. (In our previous example, D and E wind up with
the longest codes, and so they would be all the way to the right.) Among
elements with codes of the same length, those that come first in the element
set are placed to the left. (If D and E end up being the only elements with
codes of that length, then D will get the 0 branch and E the 1 branch, as D
comes before E.)

It turns out that when these two restrictions are placed upon the trees, there
is at most one possible tree for every set of elements and their respective
codelengths. The codelengths are all that we need to reconstruct the tree, and
therefore all that we need to transmit.
-->

[thuc.space Huffman]: https://thuc.space/posts/huffman_coding_algorithm/
[Abdul Bari video]: https://youtu.be/co4_ahEDCho?t=525


## Deflate

A compressed data set consists of a series of blocks of arbitrary size (except
non-compressible blocks, which are limited to 65 535 bytes) using a combination
of LZ77 and Huffman codes.

There are three modes of compression that the compressor has available:

1. Not compressed at all. This is an intelligent choice for, say, data that's
   already been compressed. Data stored in this mode will expand slightly, but
   not by as much as it would if it were already compressed and one of the
   other compression methods was tried upon it.

2. Compression, first with LZ77 and then with Huffman coding. The trees that
   are used to compress in this mode are defined by the Deflate specification
   itself, and so no extra space needs to be taken to store those trees.

3. Compression, first with LZ77 and then with Huffman coding with trees that
   the compressor creates and stores along with the data.

<!-- TODO -->
[codersnotes.com]: http://www.codersnotes.com/notes/elegance-of-deflate/
[code]: https://github.com/Frommi/miniz_oxide
[debugger]: https://github.com/madler/infgen/

