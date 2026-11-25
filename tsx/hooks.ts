// Clocks and loaders.

import { type Accessor, cleanup, effect, resource, signal, untracked } from "nsh";
import { SkData, SkFont, SkImage, SkTypeface } from "nsh-skia/native";
import { Skia } from "./Skia";

/** Milliseconds since this was called, each display frame. */
export function useClock(): Accessor<number> {
  const clock = signal(0);
  let start: number | null = null;
  let frame = 0;
  let retry = 0;
  const tick = (time: number) => {
    start ??= time;
    clock.set(time - start);
    frame = requestAnimationFrame(tick);
  };
  // On Linux (NSH nightly 2964346) a frame asked for before the window shows
  // may never come: ask again until the first one does.
  const ask = () => {
    if (start !== null) return;
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(tick);
    retry = setTimeout(ask, 250);
  };
  ask();
  cleanup(() => {
    cancelAnimationFrame(frame);
    clearTimeout(retry);
  });
  return clock;
}

/** Runs `callback` each display frame. */
export function useFrameCallback(callback: (info: { timeSinceFirstFrame: number }) => void): void {
  const clock = useClock();
  effect(() => {
    const timeSinceFirstFrame = clock();
    untracked(() => callback({ timeSinceFirstFrame }));
  });
}

/** What loaders take: bytes, or a URL `fetch` reads. */
export type DataSource = Uint8Array | ArrayBuffer | SkData | string | null | undefined;

/** Loads bytes. */
export async function loadData(source: Exclude<DataSource, null | undefined>): Promise<SkData> {
  if (source instanceof SkData) return source;
  if (typeof source === "string") return Skia.Data.fromURI(source);
  return SkData.fromBytes(source);
}

const load = <T>(
  source: DataSource | (() => DataSource),
  make: (data: SkData) => T | null,
  onError?: (error: Error) => void,
): Accessor<T | null> => {
  const from = typeof source === "function" ? source : () => source;
  const loaded = resource(from, async (s) => {
    try {
      const made = make(await loadData(s as Exclude<DataSource, null | undefined>));
      if (made === null) throw new Error("the data cannot be read");
      return made;
    } catch (e) {
      const error = e instanceof Error ? e : new Error(String(e));
      if (onError) onError(error);
      else console.error(`nsh-skia: ${error.message}`);
      return null;
    }
  });
  return () => loaded() ?? null;
};

/** An image (`null` until it is loaded). */
export const useImage = (source: DataSource | (() => DataSource), onError?: (error: Error) => void) =>
  load<SkImage>(source, (data) => SkImage.fromEncoded(data), onError);

/** A typeface (`null` until it is loaded). */
export const useTypeface = (source: DataSource | (() => DataSource), onError?: (error: Error) => void) =>
  load<SkTypeface>(source, (data) => SkTypeface.fromData(data), onError);

/** A font of a file (`null` until it is loaded). */
export const useFont = (source: DataSource | (() => DataSource), size = 14, onError?: (error: Error) => void) =>
  load<SkFont>(
    source,
    (data) => {
      const face = SkTypeface.fromData(data);
      return face && new SkFont(face, size);
    },
    onError,
  );
