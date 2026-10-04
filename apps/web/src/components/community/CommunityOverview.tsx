"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { communityApi, userSession, errorMessage, type Channel, type Community, type DemoUser, type MembershipStatus } from "@/lib/community-api";
import { CommunityWorkspace } from "./CommunityWorkspace";
import { buttonClass, ErrorNotice } from "./CommunityUi";

type Overview = { community: Community; channels: Channel[]; user: DemoUser; membership: MembershipStatus };
export function CommunityOverview({ communityKey, displayName }: { communityKey: string; displayName?: string }) {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let cancelled = false;
    async function open() {
      try {
        const community = await communityApi<Community>(`/communities/${encodeURIComponent(communityKey)}/open`, { preview: true, ...(displayName ? { name: displayName } : {}) });
        const [membership, { user }] = await Promise.all([communityApi<MembershipStatus>(`/communities/${community.id}/membership`), userSession()]);
        const channels = membership.joined ? await communityApi<Channel[]>(`/communities/${community.id}/channels`) : [];
        if (!cancelled) setOverview({ community, channels, user, membership });
      } catch (error) { if (!cancelled) setError(errorMessage(error)); }
    }
    void open(); return () => { cancelled = true; };
  }, [communityKey, displayName, attempt]);
  async function join() {
    if (!overview || busy) return;
    setBusy(true); setError("");
    try { await communityApi(`/communities/${overview.community.id}/join`, {}); setAttempt((value) => value + 1); }
    catch (error) { setError(errorMessage(error)); }
    finally { setBusy(false); }
  }
  if (!overview) return error ? <div className="p-6"><ErrorNotice message={error} onRetry={() => { setError(""); setAttempt((value) => value + 1); }} /></div> : <p role="status" className="p-8 text-sm text-[var(--site-secondary)]">Opening community…</p>;
  if (!overview.membership.joined) return <section className="mx-auto max-w-2xl p-6 md:p-12">
    <Link href="/communities" className="text-sm underline">All communities</Link>
    <h1 className="mt-8 text-3xl font-medium">{overview.community.name}</h1>
    <p className="mt-3 text-sm leading-6 text-[var(--site-secondary)]">{overview.community.description || "Connect with people in this disease community. Join to read and write posts, see members, and participate in community surveys."}</p>
    <p className="my-5 text-sm text-[var(--site-secondary)]">{overview.membership.member_count} {overview.membership.member_count === 1 ? "member" : "members"}</p>
    {error && <ErrorNotice message={error} />}
    <button type="button" onClick={() => void join()} disabled={busy} className={`${buttonClass} mt-5`}>{busy ? "Joining…" : "Join community"}</button>
  </section>;
  return <CommunityWorkspace community={overview.community} channels={overview.channels} user={overview.user} />;
}
