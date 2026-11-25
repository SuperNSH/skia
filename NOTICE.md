# Notices

nsh-skia is MIT (LICENSE). It ports and copies from:

## react-native-skia

https://github.com/wcandillon/react-native-skia at `6ef585b`. The scene renderer (`src/scene.rs`) is a port of its
recorder and player (`packages/skia/src/sksg/Recorder`); the API's names, component props and defaults, and several
small helpers (`tsx/types.ts`, `tsx/components.tsx`, `tsx/Skia.ts`) are copied or adapted from `packages/skia/src`.
`examples/gallery`'s Breathe demo follows its example of that name. Its licence:

> Copyright 2026-present, William Candillon
> Copyright 2021-2026, Shopify Inc.
> 
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
> 
> The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
> 
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## rust-skia

`vendor/skia-bindings` and `vendor/skia-safe` are [rust-skia](https://github.com/rust-skia/rust-skia) 0.157.0 (MIT), without their tests and
examples, and with `build_support/private_symbols.rs` added. Their licence: `vendor/LICENSE-rust-skia`.

## Linked, not copied

- The [Skia](https://skia.org) library rust-skia builds or downloads, BSD-3-Clause, with the third-party libraries Skia carries
  (ICU, HarfBuzz, FreeType, libpng, libjpeg-turbo, libwebp, zlib, expat).
