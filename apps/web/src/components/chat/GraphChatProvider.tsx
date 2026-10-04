"use client";

import { createContext, useContext, useEffect, useRef, useState } from "react";
import { communityApi, communityFetch, errorMessage } from "@/lib/community-api";
import { chatEvents, type ToolActivity } from "@/lib/chat-stream";
import type { GraphComplexity, GraphSnapshot } from "@/lib/graph-types";

export type ChatMessage = { role: "user" | "assistant"; content: string };
type Summary = { id: string; title: string; updated_at: string };
type Conversation = Summary & { messages: ChatMessage[]; graph: GraphSnapshot };
export const emptyGraph: GraphSnapshot = { nodes: [], edges: [], query: "", result_uid: "", truncated: false, generated_at: "" };
type ChatState = {
  graph: GraphSnapshot; messages: ChatMessage[]; tools: ToolActivity[]; chats: Summary[]; activeId: string | null;
  busy: boolean; searching: boolean; error: string; graphError: string; update: string;
  complexity: GraphComplexity; changeComplexity: (value: GraphComplexity) => Promise<void>;
  send: (message: string) => Promise<void>; stop: () => void; newChat: () => void;
  openChat: (id: string) => Promise<void>; searchDisease: (query: string) => Promise<void>;
};
const Context = createContext<ChatState | null>(null);
export function useGraphChat() { const value = useContext(Context); if (!value) throw new Error("GraphChatProvider is required"); return value; }

export function GraphChatProvider({ children }: { children: React.ReactNode }) {
  const [graph, setGraph] = useState(emptyGraph);
  const [complexity, setComplexity] = useState<GraphComplexity>("focused");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [tools, setTools] = useState<ToolActivity[]>([]);
  const [chats, setChats] = useState<Summary[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState("");
  const [graphError, setGraphError] = useState("");
  const [update, setUpdate] = useState("");
  const controller = useRef<AbortController | null>(null);
  const generation = useRef(0);
  const activeRef = useRef<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    communityApi<Summary[]>("/chats").then((items) => { if (!cancelled) setChats(items); }).catch(() => {});
    return () => { cancelled = true; controller.current?.abort(); };
  }, []);
  function newChat() {
    controller.current?.abort(); controller.current = null; generation.current++;
    activeRef.current = null; setActiveId(null); setGraph(emptyGraph); setMessages([]); setTools([]);
    setBusy(false); setSearching(false); setError(""); setGraphError(""); setUpdate("");
    setComplexity("focused");
  }
  async function searchDisease(query: string) {
    newChat();
    if (!query) return;
    const version = generation.current; setSearching(true);
    try {
      const snapshot = await communityApi<GraphSnapshot>(`/graph?${new URLSearchParams({ q: query })}`);
      if (version === generation.current) setGraph(snapshot);
    } catch (error) { if (version === generation.current) setGraphError(errorMessage(error)); }
    finally { if (version === generation.current) setSearching(false); }
  }
  async function openChat(id: string) {
    newChat(); const version = generation.current; setSearching(true);
    try {
      const chat = await communityApi<Conversation>(`/chats/${id}`);
      if (version !== generation.current) return;
      activeRef.current = id; setActiveId(id); setGraph(chat.graph); setMessages(chat.messages);
    } catch (error) { if (version === generation.current) setError(errorMessage(error)); }
    finally { if (version === generation.current) setSearching(false); }
  }
  async function changeComplexity(next: GraphComplexity) {
    if (next === complexity || controller.current || searching || !graph.nodes.length) return;
    const previous = complexity;
    const version = ++generation.current;
    setComplexity(next); setSearching(true); setGraphError(""); setUpdate("");
    try {
      const snapshot = activeRef.current
        ? (await communityApi<Conversation>(`/chats/${activeRef.current}?${new URLSearchParams({ complexity: next })}`)).graph
        : await communityApi<GraphSnapshot>(`/graph?${new URLSearchParams({ q: graph.query, complexity: next })}`);
      if (version !== generation.current) return;
      setGraph(snapshot);
      setUpdate(next === "focused" ? "Focused view restored" : "Showing more available sources and connections");
    } catch (error) {
      if (version === generation.current) { setComplexity(previous); setGraphError(errorMessage(error)); }
    } finally { if (version === generation.current) setSearching(false); }
  }
  function stop() { controller.current?.abort(); }
  async function send(message: string) {
    if (!message.trim() || controller.current || searching) return;
    const abort = new AbortController(); controller.current = abort;
    const version = generation.current;
    const initial = [...messages, { role: "user" as const, content: message }, { role: "assistant" as const, content: "" }];
    setMessages(initial); setTools([]); setBusy(true); setError(""); setUpdate("");
    let text = "";
    try {
      const response = await communityFetch("/chat", { method: "POST", signal: abort.signal, body: JSON.stringify({ conversation_id: activeRef.current, message, disease: graph.query || null, complexity }) });
      if (!response.body || !response.headers.get("content-type")?.includes("text/event-stream")) throw new Error("The backend did not return a chat stream.");
      for await (const event of chatEvents(response.body)) {
        if (version !== generation.current) break;
        switch (event.type) {
          case "conversation": activeRef.current = event.id; setActiveId(event.id); break;
          case "text": text += event.delta; setMessages([...initial.slice(0, -1), { role: "assistant", content: text }]); break;
          case "tool": setTools((items) => [...items.filter((item) => item.id !== event.activity.id), event.activity]); break;
          case "graph": setGraph(event.snapshot); setUpdate(`${event.label}${event.added ? ` · ${event.added} new sources` : " · graph updated"}`); break;
          case "error": setError(event.message); break;
          case "done": break;
        }
      }
    } catch (error) {
      if (version === generation.current) setError(abort.signal.aborted ? "Response stopped. Send a follow-up to continue." : errorMessage(error));
    } finally {
      if (version === generation.current) {
        controller.current = null; setBusy(false); setTools((items) => items.map((item) => item.status === "running" ? { ...item, status: "error", message: "Stopped before this step completed" } : item));
        communityApi<Summary[]>("/chats").then(setChats).catch(() => {});
      }
    }
  }
  return <Context.Provider value={{ graph, messages, tools, chats, activeId, busy, searching, error, graphError, update, complexity, changeComplexity, send, stop, newChat, openChat, searchDisease }}>{children}</Context.Provider>;
}
