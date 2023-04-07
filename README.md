# BinFiles

This project is a exploration of several binary formats, mainly focused on the
PNG format.

It is not intended to be efficient or bug free, this is for learning purposes
only.

# Progress

PNG

- [x] Basic chunk format
- [ ] Compression
   - [x] Filtering
   - [ ] Deflate block format
   - [ ] Huffman codes
   - [ ] LZ77
- [ ] Data structures for main chunks
   - [x] Header (`IHDR`), End (`IEND`)
   - [ ] Image data (`IDAT`)
   - [ ] Palette (`PLTE`)
   - [ ] Gamma
   - [ ] (?) Text strings
- [ ] Alpha
- [ ] Interlacing Adam7
- [ ] (?) APNG

WAV

- [x] Basic chunk format
- [x] Create WAV file with samples
- [x] Generate basic waveforms using an iterator
- [ ] Add several waveforms
- [ ] Modify existing wave files with oscillators and manually


# Other file formats to explore

- APNG (animated PNG, [APNG Wikipedia], [APNG Mozilla])
- WAV ([WAV Wikipedia], [WAV Canon], [WAV FAQ])
- PDF (?)

[APNG Wikipedia]: https://en.wikipedia.org/wiki/APNG
[APNG Mozilla]: https://wiki.mozilla.org/APNG_Specification

[WAV Wikipedia]: https://en.wikipedia.org/wiki/WAV
[WAV Canon]: http://www.lightlink.com/tjweber/StripWav/Canon.html
[WAV FAQ]:   http://www.lightlink.com/tjweber/StripWav/WAVE.html

[Tsoding PDF]: https://www.twitch.tv/videos/1750784260
[Tsoding pdf github]: https://github.com/tsoding/rust-pdf-hacking


# Useful links

- PNG inspector: [nayuki.io]
- Audacity online: [wavacity]

[nayuki.io]: https://www.nayuki.io/page/png-file-chunk-inspector
[wavacity]: https://wavacity.com/
