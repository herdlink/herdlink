"use client";

import { useRef, useState } from "react";
import { communityApi, errorMessage, type DemoUser, type Reply } from "@/lib/community-api";
import { Author, buttonClass, ErrorNotice, fieldClass, secondaryClass } from "./CommunityUi";
import { useCommunityList } from "./useCommunityList";

export function Replies({ postId, user, active }: { postId: string; user: DemoUser; active: boolean }) {
  const replies = useCommunityList<Reply>(`/threads/${postId}/comments`);
  const [body, setBody] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const submitting = useRef(false);

  async function send() {
    if (!body.trim() || submitting.current || replies.busy) return;
    submitting.current = true;
    setSending(true);
    setError("");
    setNotice("");
    try {
      const reply = await communityApi<Reply>(`/threads/${postId}/comments`, { body: body.trim() });
      replies.add(reply, "end");
      setBody("");
      setNotice("Reply sent.");
    } catch (error) { setError(errorMessage(error)); }
    finally { submitting.current = false; setSending(false); }
  }

  return (
    <section aria-label="Thread replies" className="mt-5 space-y-4 border-t border-[var(--site-border)] pt-5">
      <div className="flex items-center justify-between gap-2">
        <h4 className="text-sm font-medium">Replies</h4>
        <button type="button" onClick={replies.refresh} disabled={replies.busy || sending} className="text-xs underline underline-offset-4 disabled:opacity-40">Refresh replies</button>
      </div>
      {replies.error && <ErrorNotice message={replies.error} onRetry={replies.refresh} />}
      {replies.busy && <p role="status" className="text-xs text-[var(--site-secondary)]">Loading replies…</p>}
      {!replies.busy && !replies.error && !replies.items.length && <p className="text-sm text-[var(--site-secondary)]">No replies yet.</p>}
      {[...replies.items].sort((a, b) => a.created_at.localeCompare(b.created_at) || a.id.localeCompare(b.id)).map((reply) => (
        <div key={reply.id} className="rounded-xl bg-[var(--site-soft)] p-3">
          <Author authorId={reply.author_id} userId={user.id} createdAt={reply.created_at} />
          {reply.parent_id && <p className="mt-2 text-xs text-[var(--site-secondary)]">Reply to another comment</p>}
          <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-6">{reply.body}</p>
        </div>
      ))}
      {replies.hasMore && <button type="button" onClick={() => void replies.loadMore()} disabled={replies.busy || sending} className={secondaryClass}>Load more replies</button>}
      <form onSubmit={(event) => { event.preventDefault(); void send(); }} className="space-y-3">
        <div>
          <label htmlFor={`reply-${postId}`} className="block text-xs text-[var(--site-secondary)]">Your reply</label>
          <textarea id={`reply-${postId}`} className={`${fieldClass} mt-2 resize-y`} rows={2} maxLength={10000} required value={body} onChange={(event) => setBody(event.target.value)} disabled={sending} placeholder="Reply as Demo user…" />
        </div>
        {error && <ErrorNotice message={error} />}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span role="status" className="text-xs text-[var(--site-secondary)]">{notice}</span>
          <button type="submit" disabled={!active || !body.trim() || sending || replies.busy} className={buttonClass}>{sending ? "Sending…" : "Send reply"}</button>
        </div>
      </form>
    </section>
  );
}
