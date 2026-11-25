import { mount, signal, Text, View } from "nsh";
import { Canvas, Circle, Fill, Rect, Repeat, type CanvasRef } from "nsh-skia";

const r = signal(20);
const on = signal(false);
const info = signal("waiting");
let canvas: CanvasRef | null = null;
let only: CanvasRef | null = null;
let many: CanvasRef | null = null;
const items = signal(Array.from({ length: 1500 }, (_, i) => i));
const first = (c: CanvasRef) => Array.from(c.makeImageSnapshot().pixels().slice(0, 4)).join(",");
const ink = (c: CanvasRef) => c.makeImageSnapshot().pixels().filter((_, i) => i % 4 === 3).reduce((n, a) => n + (a ? 1 : 0), 0);

const at = (x: number, y: number) => {
  const image = canvas!.makeImageSnapshot();
  const scale = image.width() / canvas!.getSize().width;
  const i = (Math.round(y * scale) * image.width() + Math.round(x * scale)) * 4;
  return Array.from(image.pixels().slice(i, i + 4)).join(",");
};
const look = () => info.set(`${canvas!.getSize().width}x${canvas!.getSize().height} 50,50=${at(50, 50)} 85,50=${at(85, 50)} 10,10=${at(10, 10)} only=${first(only!)} many=${ink(many!)}`);

mount(() => (
    <View style={{ width: 300, height: 300 }}>
      <Canvas
        ref={(c) => (canvas = c)}
        style={{ width: 100, height: 100 }}
        onSize={() => setTimeout(look, 50)}
      >
        <Fill color="white" />
        <Circle cx={50} cy={50} r={r()} color="red" />
        {on() ? <Circle cx={10} cy={10} r={5} color="blue" /> : null}
      </Canvas>
      <Canvas ref={(c) => (only = c)} style={{ width: 10, height: 10 }}>{on() ? <Fill color="blue" /> : <Fill color="red" />}</Canvas>
      <Canvas ref={(c) => (many = c)} style={{ width: 100, height: 100 }}>
        <Repeat of={items()}>{(x: () => number) => <Rect x={x() % 100} y={Math.floor(x() / 100) * 4} width={1} height={4} color="red" />}</Repeat>
      </Canvas>
      <Text>{info()}</Text>
      <View testID="grow" onPress={() => { r.set(40); queueMicrotask(look); }}><Text>Grow</Text></View>
      <View testID="show" onPress={() => { on.set(true); items.update((l) => l.slice(0, 100)); queueMicrotask(look); }}><Text>Show</Text></View>
    </View>
));
