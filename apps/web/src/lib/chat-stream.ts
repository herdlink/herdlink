import type { GraphSnapshot } from "./graph-types";
export type ToolActivity = { id: string; name: string; label: string; status: "running" | "complete" | "error"; added?: number; message?: string };
export type ChatEvent =
  | { type: "conversation"; id: string }
  | { type: "text"; delta: string }
  | { type: "tool"; activity: ToolActivity }
  | { type: "graph"; snapshot: GraphSnapshot; label: string; added: number }
  | { type: "done" }
  | { type: "error"; message: string };

/** SSE framing is independent of network chunks, UTF-8 characters, or keepalives. */
export async function* chatEvents(body: ReadableStream<Uint8Array>): AsyncGenerator<ChatEvent> {
  const reader = body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let terminal = false;
  try {
    while (true) {
      const { done, value } = await reader.read();
      buffer += done ? decoder.decode() : decoder.decode(value, { stream: true });
      buffer = buffer.replace(/\r\n/g, "\n");
      if (buffer.length > 2_000_000) throw new Error("The assistant sent too much data in one event.");
      let end: number;
      while ((end = buffer.indexOf("\n\n")) >= 0) {
        const frame = buffer.slice(0, end); buffer = buffer.slice(end + 2);
        const lines = frame.split("\n");
        const type = lines.find((line) => line.startsWith("event:"))?.slice(6).trim();
        const data = lines.filter((line) => line.startsWith("data:")).map((line) => line.slice(5).replace(/^ /, "")).join("\n");
        if (!data) continue;
        const payload = JSON.parse(data);
        if (type === "conversation" && typeof payload.id === "string") yield { type, id: payload.id };
        else if (type === "text" && typeof payload.delta === "string") yield { type, delta: payload.delta };
        else if (type === "tool" && typeof payload.id === "string" && typeof payload.label === "string" && ["running", "complete", "error"].includes(payload.status)) yield { type, activity: payload as ToolActivity };
        else if (type === "graph" && Array.isArray(payload.snapshot?.nodes) && Array.isArray(payload.snapshot?.edges)) yield { type, snapshot: payload.snapshot as GraphSnapshot, label: String(payload.label ?? "Graph updated"), added: Number(payload.added ?? 0) };
        else if (type === "done") { terminal = true; yield { type }; }
        else if (type === "error") { terminal = true; yield { type, message: String(payload.message ?? "The response was interrupted.") }; }
      }
      if (done) break;
    }
    if (!terminal) throw new Error("The connection ended before the response finished. Send a follow-up to continue.");
  } finally { await reader.cancel().catch(() => {}); reader.releaseLock(); }
}
