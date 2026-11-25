# What is here, and what is not

Against [react-native-skia at `6ef585b`](https://github.com/wcandillon/react-native-skia/tree/6ef585b/packages/skia).

## Components

All of them, except `<Skottie>`. Differences:

| | |
|---|---|
| Shared values as props | Signals: an expression that reads one, or the accessor itself |
| `<Repeat>` | Added: `<Each>` for more than 32 rows |
| `<Group layer>` with a `<Paint>` | Read once, when the group is made |
| `blendMode` | Skia's 29. Not `plusDarker` and `plusLighter`, which react-native-skia adds |
| `<Canvas>` | `ref` (`makeImageSnapshot`, `makePicture`, `redraw`, `getSize`) and `onSize`. Not `colorSpace`, `androidWarmup`, `__destroyWebGLContextAfterRender` |

## `Skia`

Here: `Point`, `XYWHRect`, `RRectXY`, `RSXform`, `RSXformFromRadians`, `Color`, `Paint`, `Matrix`, `PictureRecorder`, `Picture`, `Path`, `PathBuilder`, `PathEffect`, `ContourMeasureIter`, `Shader`, `ColorFilter`, `MaskFilter`, `ImageFilter` (below), `RuntimeEffect`, `RuntimeShaderBuilder`, `MakeVertices`, `Data`, `Image`, `SVG`, `Font`, `FontMgr`, `Typeface`, `TypefaceFontProvider`, `ParagraphBuilder`, `TextBlob`, `Surface`.

Not here:

| Left out | Why |
|---|---|
| `Skottie` | rust-skia publishes no Skottie binaries for Apple's platforms; its properties need bindings skia-safe has not |
| `AnimatedImage` | skia-safe does not wrap `SkAnimatedImage`; to build on `Codec` |
| `ShapeBlob`, `drawShapes` | react-native-skia's own; not ported |
| `Recorder`, `getNativeDevice`, `gpu`, `MakeFromGPUTexture`, `MakeGPUTextureFromImage`, `makeTextureImage` | Dawn's. To map on nsh-webgpu by a shared device |
| `MakeFromNativeBuffer` | Not in skia-safe |
| `ImageFilter`: `MakeImage`, `MakeMagnifier`, `MakeMatrixConvolution`, `MakeMatrixTransform`, `MakeMerge`, `MakePicture`, `MakeTile`, `MakeRuntimeShaderWithChildren`, the lit ones | In skia-safe (but `MakeRuntimeShaderWithChildren`); not bound yet |
| `Font.getGlyphIntercepts`, `Paragraph.getPath`, `Paragraph.extendedVisit`, `fontVariations` | In skia-safe; not bound yet |
| `SVG.MakeFromData`, and an SVG's fonts and assets | Not bound yet |
| `Surface.MakeOffscreen` on the GPU | It is in memory |

Other differences:

- `RuntimeEffect.Make` logs the compiler's error and gives `null`; `` Sksl`...` `` throws it.
- `SkImage.encode()` and `pixels()` are synchronous, in place of `encodeToBytes()`, `encodeToBase64()` and `readPixels()`.
- `FontMgr.matchFamilyStyleNumbers(name, weight, width, slant)` in place of `matchFamilyStyle(name, style)`; `matchFont` and `matchTypeface` are as there.
- `SkSVG` is `SkSvg`.
- Every object has `dispose()`, and goes when it is collected.

## Hooks

Here: `useClock`, `useFrameCallback`, `useImage`, `useFont`, `useTypeface`. They give accessors.

Not here: `useSharedValue` and the rest of Reanimated (NSH's signals are in their place; there are no worklets, so a blocked JS thread stops the animation), `usePathInterpolation`, `usePathValue`, `useRectBuffer` and the other buffers, `useTexture`, `useFonts`, `useSVG`, `useAnimatedImage`, `useVideo`, `useCanvasRef`, `useCanvasSize` (use `onSize`).
