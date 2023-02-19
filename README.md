# PNG

This project is a exploration of the PNG format. There are comments that
summarize the [official specification].

Useful image inspector for debugging: [nayuki.io]

[official specification]: http://libpng.org/pub/png/spec/1.2/PNG-Contents.html
[nayuki.io]: https://www.nayuki.io/page/png-file-chunk-inspector

TODO: APNG (animated PNG) [APNG Wikipedia] [APNG Mozilla]

[APNG Wikipedia]: https://en.wikipedia.org/wiki/APNG
[APNG Mozilla]: https://wiki.mozilla.org/APNG_Specification

-------------------------------------------------------------------------------

# PNG Basics

The PNG format consists in a Signature and a series of chunks (PNG's read and
write operations are defined in the `png` module). This filetype is a Resource
Interchange File Format (RIFF).

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
   Comment          Miscellaneous comment; conversion from
                    GIF comment
```


# Image Layout

A PNG image is a rectangular pixel array, appearing left-to-right within each
_scanline_, and these appearing top-to-bottom.

However, the data may be transmitted in a different order, see [Interlaced][PNG Interlaced].

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
128 => -128 255 => -1
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

**Sources**: [PNG Wikipedia], [xlib.net] and [libPNG Chapter 9]

[PNG Wikipedia]: https://en.wikipedia.org/wiki/PNG#Compression
[xlib.net]: https://zlib.net/feldspar.html
[LZ77]: https://en.wikipedia.org/wiki/LZ77_and_LZ78
[Huffman coding]: https://en.wikipedia.org/wiki/Huffman_coding
[libPNG Chapter 9]: http://www.libpng.org/pub/png/book/chapter09.html
