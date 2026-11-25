// <Canvas>: shows Skia components. The scene is kept in Rust; a change
// records one picture on the JS thread, which the platform's view draws on
// its own thread (Ganesh: Metal on macOS, GL on Linux). Where there is no
// GPU view the picture is drawn in memory and shown as an image.

import { cleanup, type JSX, nativeComponent, pixelRatio, signal, split, untracked, View, type ViewProps } from "nsh";
import { SkiaHost, SkiaImage, type SkImage, type SkPicture } from "nsh-skia/native";
import { adopt, childrenOf, SceneNode } from "./scene";
import type { SkRect } from "./types";

const SkiaView = nativeComponent<{ canvas: number }>("skia.canvas", { props: ["canvas"] });

export interface SkSize {
  width: number;
  height: number;
}

/** What a `<Canvas>`'s `ref` gets. */
export interface CanvasRef {
  /** What the canvas shows, in pixels (`rect`, in points: that part). Drawn in memory. */
  makeImageSnapshot(rect?: SkRect): SkImage;
  /** What the canvas shows, as a picture. */
  makePicture(): SkPicture;
  /** Draws again. */
  redraw(): void;
  /** Its size in points (0 by 0 before its first layout). */
  getSize(): SkSize;
}

export interface CanvasProps extends Omit<ViewProps, "ref" | "children" | "onLayout"> {
  children?: JSX.Element | JSX.Element[];
  /** Gets the canvas, and `null` when it goes: a variable, a function or `{ current }`. */
  ref?: CanvasRef | null | ((canvas: CanvasRef | null) => void) | { current: CanvasRef | null };
  /** Called with the size in points, when laid out and when it changes. */
  onSize?: (size: SkSize) => void;
}

const setRef = (ref: CanvasProps["ref"], canvas: CanvasRef | null) => {
  if (typeof ref === "function") ref(canvas);
  else if (ref && "current" in ref) ref.current = canvas;
};

/** A view that shows the Skia components in it. */
export function Canvas(props: CanvasProps): JSX.Element {
  const [own, rest] = split(props, ["children", "ref", "onSize"]);
  const id = SkiaHost.allocate();
  const root = new SceneNode("Group");
  const size = signal<SkSize>({ width: 0, height: 0 });
  const frame = signal(0);
  const warned = new Set<string>();
  let queued = false;
  let gone = false;

  const present = () => {
    queued = false;
    const { width, height } = untracked(size);
    if (gone || width <= 0 || height <= 0) return;
    for (const error of root.native.present(id, width, height)) {
      if (warned.has(error)) continue;
      warned.add(error);
      console.warn(`<Canvas>: ${error}`);
    }
    frame.update((n) => n + 1);
  };
  // Every change of a turn is one frame.
  const request = () => {
    if (queued || gone) return;
    queued = true;
    queueMicrotask(present);
  };

  const canvas: CanvasRef = {
    makeImageSnapshot: (rect) => {
      const { width, height } = untracked(size);
      const scale = untracked(pixelRatio);
      const image = root.native.snapshot(width, height, scale);
      if (!rect) return image;
      const part = new SceneNode("Group");
      const drawn = new SceneNode("Image");
      drawn.set("image", image);
      drawn.set("rect", { x: -rect.x, y: -rect.y, width, height });
      part.setChildren([drawn]);
      return part.native.snapshot(rect.width, rect.height, scale);
    },
    makePicture: () => {
      const { width, height } = untracked(size);
      return root.native.picture(width, height);
    },
    redraw: request,
    getSize: () => untracked(size),
  };

  adopt(root, childrenOf(props));
  root.onChange = request;
  setRef(untracked(() => own.ref), canvas);
  cleanup(() => {
    gone = true;
    root.onChange = null;
    setRef(untracked(() => own.ref), null);
    SkiaHost.release(id);
  });

  return (
    <View {...rest}>
      <View
        style={{ flex: 1 }}
        onLayout={(e) => {
          const { width, height } = e.bounds.size;
          const last = untracked(size);
          if (gone || (last.width === width && last.height === height)) return;
          size.set({ width, height });
          own.onSize?.({ width, height });
          request();
        }}
      >
        {SkiaHost.hasGpuView() ? (
          <SkiaView canvas={id} style={{ flex: 1 }} />
        ) : (
          <SkiaImage canvas={id} frame={frame()} width={size().width} height={size().height} scale={pixelRatio()} />
        )}
      </View>
    </View>
  );
}

/** Draws components into an image, without a view. */
export function drawAsImage(element: JSX.Element, size: SkSize, scale = 1): SkImage {
  const root = new SceneNode("Group");
  adopt(root, element);
  const image = root.native.snapshot(size.width, size.height, scale);
  for (const error of root.native.errors) console.warn(`drawAsImage: ${error}`);
  return image;
}

/** Draws components into a picture, without a view. Throws what the first one that cannot be drawn says. */
export function drawAsPicture(element: JSX.Element, size: SkSize): SkPicture {
  const root = new SceneNode("Group");
  adopt(root, element);
  return root.native.picture(size.width, size.height);
}
