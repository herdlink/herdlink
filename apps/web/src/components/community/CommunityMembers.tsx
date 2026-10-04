"use client";

import { useCommunityList } from "./useCommunityList";
import { ErrorNotice, secondaryClass } from "./CommunityUi";
export function CommunityMembers({ communityId }: { communityId: string }) {
  const members = useCommunityList<{ id: string; username: string; role: string }>(`/communities/${communityId}/members`);
  return <details className="mt-3 text-xs"><summary className="cursor-pointer text-[var(--site-secondary)]">Community members</summary>
    {members.error && <ErrorNotice message={members.error} />}
    {members.busy && <p role="status" className="mt-3">Loading members…</p>}
    <ul className="mt-3 max-h-40 space-y-2 overflow-auto">{members.items.map((member) => <li key={member.id} className="flex items-center gap-2"><span className="flex h-6 w-6 items-center justify-center rounded-full bg-[var(--site-hover)]">{member.username[0]?.toUpperCase()}</span>{member.username === "demo_user" ? "Demo user" : member.username}</li>)}</ul>
    {members.hasMore && <button className={`${secondaryClass} mt-3`} disabled={members.busy} onClick={() => void members.loadMore()}>Load more members</button>}
  </details>;
}
