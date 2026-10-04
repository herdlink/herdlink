"use client";

import { useRef, useState } from "react";
import { communityApi, errorMessage, type Post } from "@/lib/community-api";
import { buttonClass, ErrorNotice, fieldClass, secondaryClass } from "./CommunityUi";

export function CreatePost({ channelId, onCreated, onCancel }: { channelId: string; onCreated: (post: Post) => void; onCancel: () => void }) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const submitting = useRef(false);

  async function publish() {
    if (!title.trim() || !body.trim() || submitting.current) return;
    submitting.current = true;
    setSending(true);
    setError("");
    try {
      const post = await communityApi<Post>(`/channels/${channelId}/threads`, { title: title.trim(), body: body.trim() });
      setTitle(""); setBody("");
      onCreated(post);
    } catch (error) { setError(errorMessage(error)); }
    finally { submitting.current = false; setSending(false); }
  }

  return (
    <section aria-labelledby="create-post-heading">
      <h2 id="create-post-heading" className="text-3xl font-medium tracking-tight">Create a post</h2>
      <p className="mb-8 mt-3 text-sm text-[var(--site-secondary)]">Share an experience or start a conversation with your community.</p>
      <form onSubmit={(event) => { event.preventDefault(); void publish(); }} className="space-y-5">
        <div><label htmlFor="new-post-title" className="mb-2 block text-sm font-medium">Title</label><input id="new-post-title" className={fieldClass} value={title} onChange={(event) => setTitle(event.target.value)} maxLength={300} required disabled={sending} placeholder="What would you like to talk about?" /></div>
        <div><label htmlFor="new-post-body" className="mb-2 block text-sm font-medium">Post</label><textarea id="new-post-body" className={`${fieldClass} resize-y`} rows={10} value={body} onChange={(event) => setBody(event.target.value)} maxLength={40000} required disabled={sending} placeholder="Write your post…" /></div>
        {error && <ErrorNotice message={error} />}
        <p className="text-xs text-[var(--site-secondary)]">Posting as Demo user · Visible to community members</p>
        <div className="flex flex-wrap justify-end gap-3"><button type="button" onClick={onCancel} disabled={sending} className={secondaryClass}>Cancel</button><button type="submit" disabled={!title.trim() || !body.trim() || sending} className={buttonClass}>{sending ? "Publishing…" : "Publish post"}</button></div>
      </form>
    </section>
  );
}
