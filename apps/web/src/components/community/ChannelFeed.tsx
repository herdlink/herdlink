"use client";

import { useRef, useState } from "react";
import { MessageCircle, RefreshCw, Send } from "lucide-react";
import { communityApi, errorMessage, type Channel, type DemoUser, type Post } from "@/lib/community-api";
import { Author, buttonClass, ErrorNotice, fieldClass, secondaryClass } from "./CommunityUi";
import { Replies } from "./Replies";
import { useCommunityList } from "./useCommunityList";

export function ChannelFeed({ channel, user, active }: { channel: Channel; user: DemoUser; active: boolean }) {
  const posts = useCommunityList<Post>(`/channels/${channel.id}/threads`);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [openReplies, setOpenReplies] = useState<string[]>([]);
  const submitting = useRef(false);

  async function send() {
    if (!body.trim() || submitting.current || posts.busy) return;
    submitting.current = true;
    setSending(true);
    setError("");
    setNotice("");
    try {
      const post = await communityApi<Post>(`/channels/${channel.id}/threads`, {
        title: title.trim() || Array.from(body.trim().split("\n")[0]).slice(0, 100).join(""),
        body: body.trim(),
      });
      posts.add(post, "start");
      setTitle("");
      setBody("");
      setNotice("Message sent.");
    } catch (error) { setError(errorMessage(error)); }
    finally { submitting.current = false; setSending(false); }
  }

  return (
    <section aria-label={`${channel.name} messages`}>
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-medium">{channel.name}</h2>
          <p className="mt-1 text-xs text-[var(--site-secondary)]">{channel.kind === "announcement" ? "Share updates with the community." : "Ask a question or start a conversation."}</p>
        </div>
        <button type="button" onClick={posts.refresh} disabled={posts.busy || sending} className={secondaryClass}><RefreshCw size={14} />Refresh</button>
      </div>
      <form onSubmit={(event) => { event.preventDefault(); void send(); }} className="mb-7 space-y-3 rounded-2xl border border-[var(--site-border)] p-4">
        <p className="text-sm font-medium">Post as Demo user</p>
        <label className="block text-xs text-[var(--site-secondary)]">Title (optional)
          <input value={title} onChange={(event) => setTitle(event.target.value)} maxLength={300} disabled={sending} placeholder="Give your conversation a title" className={`${fieldClass} mt-2`} />
        </label>
        <div>
          <label htmlFor={`message-${channel.id}`} className="block text-xs text-[var(--site-secondary)]">Message</label>
          <textarea id={`message-${channel.id}`} value={body} onChange={(event) => setBody(event.target.value)} maxLength={40000} disabled={sending} required rows={4} placeholder={`Message ${channel.name.toLowerCase()}…`} className={`${fieldClass} mt-2 resize-y`} />
        </div>
        {error && <ErrorNotice message={error} />}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <span role="status" className="text-xs text-[var(--site-secondary)]">{notice || "Visible to community members"}</span>
          <button type="submit" disabled={!body.trim() || sending || posts.busy} className={buttonClass}><Send size={14} />{sending ? "Sending…" : "Send message"}</button>
        </div>
      </form>

      {posts.error && <ErrorNotice message={posts.error} onRetry={posts.refresh} />}
      {posts.busy && <p role="status" className="py-4 text-sm text-[var(--site-secondary)]">Loading messages…</p>}
      {!posts.busy && !posts.error && !posts.items.length && (
        <div className="py-12 text-center text-[var(--site-secondary)]"><MessageCircle size={24} className="mx-auto mb-4" /><p className="text-sm">No messages yet. Start the conversation.</p></div>
      )}
      <div className="space-y-4">
        {posts.items.map((post) => {
          const expanded = openReplies.includes(post.id);
          return (
            <article key={post.id} className="min-w-0 rounded-2xl border border-[var(--site-border)] p-4 md:p-5">
              <Author authorId={post.author_id} userId={user.id} createdAt={post.created_at} />
              <h3 className="mt-3 break-words text-base font-medium">{post.title}</h3>
              <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-6">{post.body}</p>
              <button type="button" aria-expanded={expanded} aria-controls={`replies-${post.id}`} onClick={() => setOpenReplies((current) => expanded ? current.filter((id) => id !== post.id) : [...current, post.id])} className={`${secondaryClass} mt-4`}><MessageCircle size={14} />{expanded ? "Hide replies" : "Replies"}</button>
              <div id={`replies-${post.id}`} hidden={!expanded}>
                {expanded && <Replies postId={post.id} user={user} active={active} />}
              </div>
            </article>
          );
        })}
      </div>
      {posts.hasMore && <button type="button" onClick={() => void posts.loadMore()} disabled={posts.busy || sending} className={`${secondaryClass} mt-5`}>Load older messages</button>}
    </section>
  );
}
