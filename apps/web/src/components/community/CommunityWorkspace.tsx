"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { ArrowLeft, ChevronRight, FileText, Megaphone, Plus, RefreshCw, Search, X } from "lucide-react";
import { type Channel, type Community, type DemoUser, type Post } from "@/lib/community-api";
import { CommunityMembers } from "./CommunityMembers";
import { ChannelFeed } from "./ChannelFeed";
import { Author, buttonClass, ErrorNotice, secondaryClass } from "./CommunityUi";
import { CreatePost } from "./CreatePost";
import { Replies } from "./Replies";
import { useCommunityList } from "./useCommunityList";

type Selection = { kind: "announcements" } | { kind: "new" } | { kind: "post"; post: Post };

export function CommunityWorkspace({ community, channels, user }: { community: Community; channels: Channel[]; user: DemoUser }) {
  const announcements = channels.find((channel) => channel.kind === "announcement");
  const discussions = channels.find((channel) => channel.kind === "discussion");
  const [selection, setSelection] = useState<Selection>({ kind: "announcements" });
  const [query, setQuery] = useState("");
  const [search, setSearch] = useState("");
  const [revision, setRevision] = useState(0);
  const [showContent, setShowContent] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const timer = setTimeout(() => setSearch(query.trim()), 250);
    return () => clearTimeout(timer);
  }, [query]);

  function select(next: Selection) {
    setSelection(next);
    setShowContent(true);
    requestAnimationFrame(() => {
      contentRef.current?.scrollTo({ top: 0 });
      contentRef.current?.focus({ preventScroll: true });
    });
  }

  return (
    <div className="grid h-full min-h-0 grid-cols-1 md:grid-cols-[280px_minmax(0,1fr)] lg:grid-cols-[320px_minmax(0,1fr)]">
      <aside aria-label="Community sidebar" className={`${showContent ? "hidden md:flex" : "flex"} min-h-0 min-w-0 flex-col border-r border-[var(--site-border)]`}>
        <header className="shrink-0 px-5 pb-5 pt-6">
          <Link href="/communities" className="mb-5 inline-flex items-center gap-2 text-xs text-[var(--site-secondary)] hover:text-[var(--site-text)]"><ArrowLeft size={14} />All communities</Link>
          <h1 className="break-words text-xl font-medium tracking-tight">{community.name}</h1>
          <p className="mt-2 text-xs text-[var(--site-secondary)]">You’re a member</p>
          <CommunityMembers communityId={community.id} />
          <Link href={{ pathname: "/surveys", query: { community: community.id } }} className="mt-4 inline-flex text-xs underline">Community surveys</Link>
        </header>
        <div className="shrink-0 px-3 pb-4">
          <button type="button" aria-pressed={selection.kind === "announcements"} onClick={() => select({ kind: "announcements" })} className={`flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm transition-colors ${selection.kind === "announcements" ? "bg-[var(--site-hover)] font-medium" : "hover:bg-[var(--site-soft)]"}`}>
            <Megaphone size={18} /><span className="flex-1">Announcements</span><ChevronRight size={14} className="text-[var(--site-secondary)]" />
          </button>
        </div>
        <div className="shrink-0 space-y-4 border-t border-[var(--site-border)] px-5 pb-4 pt-5">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-sm font-semibold">Posts</h2>
            <button type="button" onClick={() => select({ kind: "new" })} disabled={!discussions} className={`${buttonClass} px-3`}><Plus size={14} />New post</button>
          </div>
          <div className="flex items-center gap-2 rounded-xl border border-[var(--site-border)] bg-[var(--site-soft)] px-3 focus-within:ring-1 focus-within:ring-[var(--site-secondary)]">
            <Search size={16} className="shrink-0 text-[var(--site-secondary)]" />
            <input ref={searchRef} type="search" aria-label="Search community posts" placeholder="Search posts…" value={query} onChange={(event) => setQuery(event.target.value)} maxLength={200} onKeyDown={(event) => { if (event.key === "Escape") { setQuery(""); setSearch(""); } }} className="min-w-0 flex-1 appearance-none bg-transparent py-3 text-sm outline-none [&::-webkit-search-cancel-button]:hidden" />
            {query && <button type="button" aria-label="Clear search" onClick={() => { setQuery(""); setSearch(""); searchRef.current?.focus(); }} className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><X size={14} /></button>}
          </div>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-5">
          {discussions && <PostList key={`${search}:${revision}`} channelId={discussions.id} query={search} user={user} selectedId={selection.kind === "post" ? selection.post.id : null} onSelect={(post) => select({ kind: "post", post })} />}
        </div>
      </aside>

      <section aria-label="Community content" className={`${showContent ? "flex" : "hidden md:flex"} min-h-0 min-w-0 flex-col`}>
        <header className="flex h-16 shrink-0 items-center gap-3 border-b border-[var(--site-border)] px-5 md:px-8">
          <button type="button" onClick={() => setShowContent(false)} className="flex items-center gap-2 rounded-full py-2 pr-2 text-sm md:hidden"><ArrowLeft size={16} />Posts</button>
          <span className="truncate text-sm text-[var(--site-secondary)]">{selection.kind === "announcements" ? "Announcements" : selection.kind === "new" ? "Create a post" : "Community post"}</span>
        </header>
        <div ref={contentRef} tabIndex={-1} className="min-h-0 flex-1 overflow-y-auto focus:outline-none">
          <div className="mx-auto w-full max-w-4xl p-5 md:p-8 lg:p-12">
            <div hidden={selection.kind !== "announcements"}>
              {announcements && <ChannelFeed channel={announcements} user={user} active={selection.kind === "announcements"} />}
            </div>
            <div hidden={selection.kind !== "new"}>
              {discussions && <CreatePost channelId={discussions.id} onCreated={(post) => {
                setQuery(""); setSearch(""); setRevision((value) => value + 1);
                select({ kind: "post", post });
              }} onCancel={() => select({ kind: "announcements" })} />}
            </div>
            {selection.kind === "post" && (
              <article key={selection.post.id}>
                <Author authorId={selection.post.author_id} userId={user.id} createdAt={selection.post.created_at} />
                <h2 className="mt-5 break-words text-3xl font-medium leading-tight tracking-tight">{selection.post.title}</h2>
                <p className="mb-10 mt-6 whitespace-pre-wrap break-words text-base leading-7">{selection.post.body}</p>
                <Replies postId={selection.post.id} user={user} active />
              </article>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}

function PostList({ channelId, query, selectedId, user, onSelect }: { channelId: string; query: string; selectedId: string | null; user: DemoUser; onSelect: (post: Post) => void }) {
  const posts = useCommunityList<Post>(`/channels/${channelId}/threads?q=${encodeURIComponent(query)}`);
  return (
    <>
      <div className="mb-2 flex items-center justify-between gap-2 px-2">
        <span role="status" className="text-xs text-[var(--site-secondary)]">{posts.busy ? "Loading posts…" : query ? `${posts.items.length}${posts.hasMore ? "+" : ""} results` : "Recent posts"}</span>
        <button type="button" aria-label="Refresh posts" onClick={posts.refresh} disabled={posts.busy} className="flex h-8 w-8 items-center justify-center rounded-full text-[var(--site-secondary)] hover:bg-[var(--site-hover)] disabled:opacity-40"><RefreshCw size={14} /></button>
      </div>
      {posts.error && <ErrorNotice message={posts.error} onRetry={posts.refresh} />}
      {!posts.busy && !posts.error && !posts.items.length && <div className="px-3 py-8 text-center text-sm text-[var(--site-secondary)]"><FileText size={22} className="mx-auto mb-3" /><p>{query ? "No posts match your search." : "No posts yet. Start the first conversation."}</p></div>}
      <ul aria-label="Community posts" className="space-y-1">
        {posts.items.map((post) => (
          <li key={post.id}>
            <button type="button" onClick={() => onSelect(post)} aria-current={selectedId === post.id ? "true" : undefined} className={`w-full rounded-xl p-3 text-left transition-colors ${selectedId === post.id ? "bg-[var(--site-hover)]" : "hover:bg-[var(--site-soft)]"}`}>
              <span className="line-clamp-2 break-words text-sm font-medium"><Highlighted text={post.title} query={query} /></span>
              <span className="mt-1.5 line-clamp-2 break-words text-xs leading-5 text-[var(--site-secondary)]"><Highlighted text={post.body} query={query} /></span>
              <span className="mt-2 block text-[11px] text-[var(--site-secondary)]">{post.author_id === user.id ? "You" : `Member ${post.author_id.slice(0, 8)}`} · {new Date(post.created_at).toLocaleDateString(undefined, { month: "short", day: "numeric" })}</span>
            </button>
          </li>
        ))}
      </ul>
      {posts.hasMore && <button type="button" onClick={() => void posts.loadMore()} disabled={posts.busy} className={`${secondaryClass} mt-4 w-full`}>Load more posts</button>}
    </>
  );
}

function Highlighted({ text, query }: { text: string; query: string }) {
  const index = query ? text.toLowerCase().indexOf(query.toLowerCase()) : -1;
  if (index < 0) return text;
  return <>{text.slice(0, index)}<mark className="rounded-sm bg-[var(--site-hover)] text-[var(--site-heading)] underline decoration-[var(--site-secondary)] underline-offset-2">{text.slice(index, index + query.length)}</mark>{text.slice(index + query.length)}</>;
}
