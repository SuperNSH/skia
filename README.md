# nsh-skia

[Skia](https://skia.org) for Native Shell (NSH) apps: [react-native-skia](https://github.com/wcandillon/react-native-skia)'s API on [skia-safe](https://github.com/rust-skia/rust-skia).

The same names: `<Canvas>` and its components (`<Circle>`, `<Path>`, `<LinearGradient>`, `<Blur>`, `<Mask>`, ...), and the imperative `Skia` object (`Skia.Path`, `Skia.Paint()`, `Skia.RuntimeEffect`, ...). NSH is not React, so there is no reconciler: each component makes a node kept in Rust when it runs, a signal changes one prop of one node, and Rust records the frame.

## Status

**0.1.0, on GitHub (not on crates.io). macOS and Linux.** It needs NSH 0.7.

| Platform | Canvas | Checked |
|---|---|---|
| macOS | Ganesh on Metal, on a thread of its own | Headless tests; the gallery in a window on Apple silicon |
| Linux | Ganesh on GL, in a `GtkGLArea` | Headless tests; the gallery in a window on X11 and on Wayland (weston), arm64, with a software GPU (llvmpipe) |
| iOS, Android, Windows | No GPU view yet: drawn in memory and shown as an image | Not built or run |
| Web | None: Skia has no build for NSH's web target | |

The same 48 scenes are drawn by nsh-skia and by react-native-skia itself (on CanvasKit, in Node) and compared: the 46 its web build can draw differ by at most 2 of 255 in any channel of any pixel (`docs/evidence/parity`).

Not covered: text in that comparison (system fonts differ), a real Linux GPU, x86-64 Linux in a window, resizing under load, leaks over long runs. What of react-native-skia's API is left out is in [docs/parity.md](docs/parity.md).

## Install

Rust, and NSH 0.7. Then, in the app's directory:

```sh
nsh add github:SuperNSH/skia
# or, from a clone
nsh add --path ../skia
```

Skia comes prebuilt from rust-skia's releases (18 MB on macOS, 26 MB on Linux) when the crate is built (`vendor/` holds its two crates, for one change: see docs/architecture.md); it adds about 17 to 20 MB to an app. On Linux the build needs `libgl-dev libegl-dev libgles-dev libwayland-dev libx11-dev libfontconfig1-dev libfreetype-dev` beside NSH's own packages, and glibc 2.35 or newer.

## Use

```tsx
import { computed, mount } from "nsh";
import { BlurMask, Canvas, Circle, Fill, Group, LinearGradient, useClock, vec } from "nsh-skia";

function App() {
  const clock = useClock();
  const r = computed(() => 60 + 20 * Math.sin(clock() / 500));
  return (
    <Canvas style={{ width: 256, height: 256 }}>
      <Fill>
        <LinearGradient start={vec(0, 0)} end={vec(256, 256)} colors={["#0ea5e9", "#a855f7"]} />
      </Fill>
      <Circle cx={128} cy={128} r={r()} color="white">
        <BlurMask blur={8} style="solid" />
      </Circle>
    </Canvas>
  );
}

mount(App);
```

A prop follows what its expression reads (`r={r()}`), and takes an accessor too (`r={r}`). Where react-native-skia takes a Reanimated shared value, use a signal or a `computed`.

Conditions (`{on() ? <A /> : null}`, `<If>`) and `<Each>` work inside a `<Canvas>`. For more than 32 rows use `<Repeat>`, which takes the same props: NSH renders an `<Each>`'s rows past the 32nd when they come on screen, which a scene's never do.

```tsx
import { Skia, rect } from "nsh-skia";

const surface = Skia.Surface.MakeOffscreen(256, 256)!;
const paint = Skia.Paint();
paint.setColor(Skia.Color("tomato"));
surface.getCanvas().drawRRect({ rect: rect(16, 16, 224, 224), rx: 32, ry: 32 }, paint);
const png = surface.makeImageSnapshot().encode();
```

## Layout

- `src/`: the engine. `objects.rs`, `paint.rs`, `path.rs`, `canvas.rs`, `paragraph.rs`: Skia's objects as NSH classes. `scene.rs`: the declarative scene. `view/`: the canvas view of each platform.
- `tsx/`: the API apps import.
- `examples/headless`: the tests. `examples/gallery`: eight canvases in a window. `examples/parity` with `tools/parity`: the comparison with react-native-skia.
- [docs/architecture.md](docs/architecture.md), [docs/parity.md](docs/parity.md), `docs/evidence/`.

## Checks

```sh
just gates
```

Format and Clippy, then `nsh check`, `nsh build` and `nsh test` in `examples/headless` and `examples/parity`. `just compare <react-native-skia checkout>` runs the comparison.

## Licence

MIT. [NOTICE.md](NOTICE.md) lists what is ported from react-native-skia (MIT) and what is linked.
