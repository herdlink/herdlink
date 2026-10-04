"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { ArrowRight, Users } from "lucide-react";
import { type Community } from "@/lib/community-api";
import { buttonClass, ErrorNotice, fieldClass, secondaryClass } from "./CommunityUi";
import { useCommunityList } from "./useCommunityList";

export function CommunityDirectory() {
  const router = useRouter();
  const communities = useCommunityList<Community>("/communities");
  const [communityKey, setCommunityKey] = useState("");

  return (
    <section className="mx-auto max-w-5xl p-5 md:p-8">
      <div className="mb-8 flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-3xl font-medium tracking-tight">Communities</h1>
      </div>
      <p className="mb-6 text-sm leading-6 text-[var(--site-secondary)]">Find your community. Share experiences, ask questions, and join the conversation.</p>
      <form onSubmit={(event) => { event.preventDefault(); if (communityKey.trim()) router.push(`/community/${encodeURIComponent(communityKey.trim().toLowerCase())}`); }} className="mb-8 space-y-3 rounded-2xl border border-[var(--site-border)] p-4">
        <label className="block text-sm font-medium" htmlFor="community-key">Open a community</label>
        <div className="flex flex-col gap-3 sm:flex-row">
          <input id="community-key" className={`${fieldClass} min-w-0 flex-1`} value={communityKey} onChange={(event) => setCommunityKey(event.target.value)} placeholder="e.g. wilsons-disease" required minLength={1} maxLength={80} pattern="[a-zA-Z0-9]+(-[a-zA-Z0-9]+)*" title="Use letters, numbers, and single hyphens, or a UUID." />
          <button type="submit" disabled={!communityKey.trim()} className={buttonClass}>Open<ArrowRight size={15} /></button>
        </div>
        <p className="text-xs leading-5 text-[var(--site-secondary)]">Enter a community ID or name using hyphens. If it doesn’t exist, it will be created when you open it.</p>
      </form>
      <div className="mb-5 flex items-center justify-between gap-3">
        <h2 className="text-lg font-medium">All communities</h2>
        <button type="button" onClick={communities.refresh} disabled={communities.busy} className={secondaryClass}>Refresh</button>
      </div>
      {communities.error && <ErrorNotice message={communities.error} onRetry={communities.refresh} />}
      {communities.busy && <p role="status" className="py-5 text-sm text-[var(--site-secondary)]">Loading communities…</p>}
      {!communities.busy && !communities.error && !communities.items.length && <p className="py-8 text-sm text-[var(--site-secondary)]">No communities yet. Open one above to get started.</p>}
      <div className="space-y-3">
        {communities.items.map((community) => (
          <Link key={community.id} href={`/community/${community.id}`} className="flex items-start gap-4 rounded-2xl border border-[var(--site-border)] p-5 hover:bg-[var(--site-soft)]">
            <Users size={20} className="mt-1 shrink-0 text-[var(--site-secondary)]" />
            <div className="min-w-0 flex-1"><h3 className="break-words font-medium">{community.name}</h3><p className="mt-1 break-all text-xs text-[var(--site-secondary)]">{community.slug}</p>{community.description && <p className="mt-3 line-clamp-2 text-sm text-[var(--site-secondary)]">{community.description}</p>}</div>
            <ArrowRight size={16} className="mt-1 shrink-0" />
          </Link>
        ))}
      </div>
      {communities.hasMore && <button type="button" onClick={() => void communities.loadMore()} disabled={communities.busy} className={`${secondaryClass} mt-5`}>Load more communities</button>}
    </section>
  );
}
