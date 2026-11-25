// The gallery: one canvas for each thing nsh-skia draws, most of them
// moving. The header's numbers are the evidence: frames the canvas views
// put on screen, and the process's CPU time.

import { cleanup, mount, platform, signal, Text as Label, View } from "nsh";
import { Canvas } from "nsh-skia";
import { SkiaHost } from "nsh-skia/native";
import { GalleryHost } from "skia-gallery/native";
import { DEMOS, S } from "./demos";

const info = signal("measuring");
let last = { frames: 0, cpu: 0, time: 0 };
let ticks = 0;
const timer = setInterval(() => {
  const now = { frames: SkiaHost.frames(), cpu: GalleryHost.cpu(), time: performance.now() };
  const seconds = (now.time - last.time) / 1000;
  if (last.time > 0) {
    const perCanvas = (now.frames - last.frames) / seconds / (DEMOS.length - 1); // Shadows does not move.
    info.set(`${platform.os}: ${perCanvas.toFixed(1)} frames/s per moving canvas, CPU ${((now.cpu - last.cpu) / seconds / 10).toFixed(0)}%`);
    console.log(`STAT ${info()}`);
  }
  last = now;
  if (++ticks === 5) GalleryHost.capture("gallery-1.png");
  if (ticks === 6) GalleryHost.capture("gallery-2.png");
}, 2000);
cleanup(() => clearInterval(timer));

mount(() => (
  <View style={{ flex: 1, padding: 16, gap: 12, backgroundColor: "#111827" }}>
    <Label style={{ color: "white", fontSize: 15 }}>{`nsh-skia gallery — ${info()}`}</Label>
    <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 12 }}>
      <Each of={DEMOS}>
        {(demo: any) => {
          const Demo = demo()[1];
          return (
            <View style={{ gap: 4 }}>
              <Canvas style={{ width: S, height: S, borderRadius: 12, overflow: "hidden" }}><Demo /></Canvas>
              <Label style={{ color: "#9ca3af", fontSize: 12 }}>{demo()[0]}</Label>
            </View>
          );
        }}
      </Each>
    </View>
  </View>
));
