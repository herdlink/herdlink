import { expect, test } from "bun:test";
import { chatEvents } from "./chat-stream";
function fragmented(text: string) {
  const bytes = new TextEncoder().encode(text);
  return new ReadableStream<Uint8Array>({ start(controller) { for (const byte of bytes) controller.enqueue(new Uint8Array([byte])); controller.close(); } });
}
test("SSE decoding survives fragmented UTF-8, CRLF, keepalives and interleaved graph events", async () => {
  const wire = ': keepalive\r\n\r\nevent: conversation\r\ndata: {"id":"chat"}\r\n\r\nevent: text\r\ndata: {"delta":"Héllo 🌱"}\r\n\r\nevent: tool\r\ndata: {"id":"call","name":"pubtator_relations","label":"Find genes","status":"running"}\r\n\r\nevent: graph\r\ndata: {"snapshot":{"nodes":[],"edges":[],"query":"Huntington","result_uid":"","truncated":false,"generated_at":""},"label":"Find genes","added":0}\r\n\r\nevent: done\r\ndata: {}\r\n\r\n';
  const events = [];
  for await (const event of chatEvents(fragmented(wire))) events.push(event);
  expect(events.map((event) => event.type)).toEqual(["conversation", "text", "tool", "graph", "done"]);
  expect(events[1]).toEqual({ type: "text", delta: "Héllo 🌱" });
});
test("premature EOF is an error, but an explicit backend error is returned to the chat", async () => {
  async function collect(text: string) { const items = []; for await (const event of chatEvents(fragmented(text))) items.push(event); return items; }
  await expect(collect('event: text\ndata: {"delta":"partial"}\n\n')).rejects.toThrow("before the response finished");
  expect(await collect('event: error\ndata: {"message":"Unavailable"}\n\n')).toEqual([{ type: "error", message: "Unavailable" }]);
});
test("closing the consumer cancels the underlying streaming request", async () => {
  let cancelled = false;
  const body = new ReadableStream<Uint8Array>({ start(controller) { controller.enqueue(new TextEncoder().encode('event: text\ndata: {"delta":"first"}\n\n')); }, cancel() { cancelled = true; } });
  for await (const event of chatEvents(body)) { expect(event.type).toBe("text"); break; }
  expect(cancelled).toBe(true);
});
