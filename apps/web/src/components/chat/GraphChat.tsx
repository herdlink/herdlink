"use client";
import { useEffect, useRef, useState } from "react";
import { ArrowUp, Check, Loader2, Plus, Square } from "lucide-react";
import { useGraphChat } from "./GraphChatProvider";

function ChatText({ text }: { text: string }) {
  const parts = text.split(/(\[[^\]]+\]\(https?:\/\/[^\s)]+\)|\*\*[^*]+\*\*|`[^`]+`|PMID\s*:?\s*\d+)/g);
  return <p className="whitespace-pre-wrap break-words text-[13px] leading-6">{parts.map((part, i) => {
    const link = /^\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)$/.exec(part);
    if (link) return <a key={i} href={link[2]} target="_blank" rel="noopener noreferrer" className="underline underline-offset-4">{link[1]}</a>;
    if (part.startsWith("**") && part.endsWith("**")) return <strong key={i} className="font-semibold">{part.slice(2, -2)}</strong>;
    if (part.startsWith("`") && part.endsWith("`")) return <code key={i} className="rounded bg-[var(--site-soft)] px-1 text-xs">{part.slice(1, -1)}</code>;
    const pmid = /^PMID\s*:?\s*(\d+)$/.exec(part);
    return pmid ? <a key={i} href={`https://pubmed.ncbi.nlm.nih.gov/${pmid[1]}/`} target="_blank" rel="noopener noreferrer" className="underline underline-offset-4">{part}</a> : part;
  })}</p>;
}
export function GraphChat() {
  const chat = useGraphChat();
  const [draft, setDraft] = useState("");
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => { bottom.current?.scrollIntoView({ block: "nearest" }); }, [chat.messages, chat.tools]);
  function submit() { if (!draft.trim() || chat.busy || chat.searching) return; const message = draft.trim(); setDraft(""); void chat.send(message); }
  const prompts = ["Find related diseases based on similar genes", "Compare diseases with similar phenotypes", "Find papers explaining these connections"];
  return <>
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-[var(--site-border)] px-5">
      <h2 id="chat-heading" className="text-[14px] font-semibold">Herdlink assistant</h2>
      <button type="button" onClick={chat.newChat} aria-label="New chat" title="New chat" className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><Plus size={16} /></button>
    </header>
    <div className="min-h-0 flex-1 overflow-y-auto px-5 py-5">
      {chat.messages.length === 0 && <div className="space-y-4 text-[13px] leading-6 text-[var(--site-secondary)]"><p>Explore a disease through its genes, phenotypes, and literature.</p><p className="text-xs">Search for a disease in the graph, or name one in your message.</p><div className="space-y-2">{prompts.map((prompt) => <button key={prompt} onClick={() => setDraft(prompt)} className="block w-full rounded-xl border border-[var(--site-border)] px-3 py-2 text-left text-xs leading-5 hover:bg-[var(--site-hover)]">{prompt}</button>)}</div></div>}
      <div role="log" aria-label="Chat messages" aria-live="polite" aria-relevant="additions text" className="space-y-6">{chat.messages.map((message, i) => <article key={i} className={message.role === "user" ? "rounded-2xl bg-[var(--site-soft)] px-3 py-2" : ""}><p className="mb-1 text-[10px] font-medium uppercase tracking-wider text-[var(--site-secondary)]">{message.role === "user" ? "You" : "Herdlink"}</p>{message.content ? <ChatText text={message.content} /> : chat.busy && <p role="status" className="flex items-center gap-2 text-xs text-[var(--site-secondary)]"><Loader2 size={13} className="animate-spin" />Thinking…</p>}</article>)}</div>
      {chat.tools.length > 0 && <details open className="mt-5 rounded-xl border border-[var(--site-border)] p-3"><summary className="cursor-pointer text-xs font-medium">Research activity</summary><ul className="mt-3 space-y-3">{chat.tools.map((tool) => <li key={tool.id} className="flex gap-2 text-[11px] leading-5 text-[var(--site-secondary)]">{tool.status === "running" ? <Loader2 size={13} className="mt-1 shrink-0 animate-spin" /> : tool.status === "complete" ? <Check size={13} className="mt-1 shrink-0" /> : <span aria-hidden="true">!</span>}<span>{tool.label}{tool.status === "running" ? "…" : tool.status === "error" ? ` · ${tool.message ?? "unavailable"}` : tool.added ? ` · ${tool.added} new sources` : " · complete"}</span></li>)}</ul></details>}
      {chat.error && <p role="alert" className="mt-4 text-xs leading-5 text-[var(--site-secondary)]">{chat.error}</p>}
      <div ref={bottom} />
    </div>
    <form onSubmit={(event) => { event.preventDefault(); submit(); }} className="m-4 rounded-2xl border border-[var(--site-border)] bg-[var(--site-soft)] p-3">
      <textarea aria-label="Message Herdlink assistant" value={draft} onChange={(event) => setDraft(event.target.value)} placeholder="Ask about genes, phenotypes, or papers…" rows={3} maxLength={4000} onKeyDown={(event) => { if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) { event.preventDefault(); submit(); } }} className="block w-full resize-none bg-transparent text-sm leading-6 outline-none placeholder:text-[var(--site-secondary)]" />
      <div className="mt-2 flex items-center justify-between gap-2"><span className="text-[10px] text-[var(--site-secondary)]">{chat.busy ? "Researching…" : "Enter to send · Shift+Enter for a new line"}</span>{chat.busy ? <button type="button" onClick={chat.stop} aria-label="Stop response" className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--site-solid)] text-[var(--site-solid-text)]"><Square size={12} fill="currentColor" /></button> : <button type="submit" aria-label="Send message" disabled={!draft.trim() || chat.searching} className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--site-solid)] text-[var(--site-solid-text)] disabled:opacity-30"><ArrowUp size={17} /></button>}</div>
    </form>
  </>;
}
export function ChatHistory() {
  const chat = useGraphChat();
  return <div className="h-full overflow-y-auto px-3 pb-5"><button type="button" onClick={chat.newChat} className="mb-3 flex w-full items-center gap-2 rounded-full border border-[var(--site-border)] px-3 py-2 text-xs hover:bg-[var(--site-hover)]"><Plus size={13} />New conversation</button><ul className="space-y-1">{chat.chats.map((item) => <li key={item.id}><button type="button" onClick={() => void chat.openChat(item.id)} aria-current={item.id === chat.activeId ? "true" : undefined} className={`w-full truncate rounded-xl px-3 py-2.5 text-left text-xs hover:bg-[var(--site-hover)] ${item.id === chat.activeId ? "bg-[var(--site-hover)] font-medium" : "text-[var(--site-secondary)]"}`} title={item.title}>{item.title}</button></li>)}</ul>{chat.chats.length === 0 && <p className="px-3 py-2 text-xs leading-5 text-[var(--site-secondary)]">Your conversations will appear here.</p>}</div>;
}
