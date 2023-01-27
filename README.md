# PNG

This project is a exploration of the PNG format. There are comments that
summarize the [official specification][os].

Useful image inspector for debugging: [nayuki.io][ins]

[os]: http://libpng.org/pub/png/spec/1.2/PNG-Contents.html
[ins]: https://www.nayuki.io/page/png-file-chunk-inspector

-------------------------------------------------------------------------------

# PNG Basics

The PNG format consists in a Signature and a series of chunks (PNG's read and
write operations are defined in the `png` module).

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

However, the data may be transmitted in a different order, see [Interlaced][i].

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

[i]: http://libpng.org/pub/png/spec/1.2/PNG-DataRep.html#DR.Interlaced-data-order

## Alpha
## Interlaced
## Gamma
# Text Strings
# Compression
# Filter algorithms

