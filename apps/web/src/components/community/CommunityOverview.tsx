"use client";

import { useEffect, useState } from "react";
import { communityApi, demoSession, errorMessage, type Channel, type Community, type DemoUser } from "@/lib/community-api";
import { CommunityWorkspace } from "./CommunityWorkspace";
import { ErrorNotice } from "./CommunityUi";

type Overview = { community: Community; channels: Channel[]; user: DemoUser };

export function CommunityOverview({ communityKey, displayName }: { communityKey: string; displayName?: string }) {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let cancelled = false;
    async function open() {
      try {
        const community = await communityApi<Community>(`/communities/${encodeURIComponent(communityKey)}/open`, displayName ? { name: displayName } : {});
        const [channels, { user }] = await Promise.all([
          communityApi<Channel[]>(`/communities/${community.id}/channels`), demoSession(),
        ]);
        if (!cancelled) setOverview({ community, channels, user });
      } catch (error) {
        if (!cancelled) setError(errorMessage(error));
      }
    }
    void open();
    return () => { cancelled = true; };
  }, [communityKey, displayName, attempt]);

  if (error) return <div className="p-6"><ErrorNotice message={error} onRetry={() => { setError(""); setAttempt((value) => value + 1); }} /></div>;
  if (!overview) return <p role="status" className="p-8 text-sm text-[var(--site-secondary)]">Opening community…</p>;
  return <CommunityWorkspace {...overview} />;
}
