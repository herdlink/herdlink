"use client";

import Link from "next/link";
import { useGraphChat } from "@/components/chat/GraphChatProvider";
import { useCommunityList } from "@/components/community/useCommunityList";
import { ErrorNotice, secondaryClass } from "@/components/community/CommunityUi";
import { graphSurveyTargets, type Survey } from "@/lib/survey-types";
import { CreateSurveyButton } from "./CreateSurvey";

export function SurveyList({ community }: { community?: string }) {
  const { graph } = useGraphChat();
  const surveys = useCommunityList<Survey>(`/surveys${community ? `?community=${encodeURIComponent(community)}` : ""}`);
  return <section className="mx-auto max-w-5xl p-5 md:p-8">
    <header className="mb-6 flex flex-wrap items-center justify-between gap-3"><div><h1 className="text-3xl font-medium">Surveys</h1><p className="mt-2 text-sm text-[var(--site-secondary)]">Surveys you created and surveys available in your communities.</p></div><CreateSurveyButton targets={graphSurveyTargets(graph)} /></header>
    <p className="mb-6 text-sm leading-6 text-[var(--site-secondary)]">Create a survey for disease communities in your current graph. <Link href="/" className="underline">Return to the graph</Link> to choose diseases.</p>
    {community && <Link href="/surveys" className="mb-5 inline-flex text-sm underline">Show all my surveys</Link>}
    {surveys.error && <ErrorNotice message={surveys.error} onRetry={surveys.refresh} />}
    {surveys.busy && <p role="status" className="py-5 text-sm">Loading surveys…</p>}
    {!surveys.busy && !surveys.error && !surveys.items.length && <div className="rounded-2xl border border-[var(--site-border)] p-8 text-sm text-[var(--site-secondary)]">No surveys yet. Join a disease community to participate, or create a survey from your graph.</div>}
    <div className="space-y-3">{surveys.items.map((survey) => <Link href={`/surveys/${survey.id}`} key={survey.id} className="block rounded-2xl border border-[var(--site-border)] p-5 hover:bg-[var(--site-soft)]"><div className="flex flex-wrap items-center justify-between gap-3"><h2 className="font-medium">{survey.title}</h2><span className="rounded-full bg-[var(--site-soft)] px-3 py-1 text-xs">{survey.answers ? "Answered" : survey.can_respond ? "Open to you" : "Created by you"}</span></div>{survey.description && <p className="mt-2 line-clamp-2 text-sm text-[var(--site-secondary)]">{survey.description}</p>}<p className="mt-3 text-xs text-[var(--site-secondary)]">{survey.communities.map((c) => c.name).join(" · ")}</p><p className="mt-2 text-xs text-[var(--site-secondary)]">{survey.response_count} responses · {survey.audience_count} unique members</p></Link>)}</div>
    <div className="mt-6 flex gap-3"><button className={secondaryClass} disabled={surveys.busy} onClick={surveys.refresh}>Refresh</button>{surveys.hasMore && <button className={secondaryClass} disabled={surveys.busy} onClick={() => void surveys.loadMore()}>Load more surveys</button>}</div>
  </section>;
}
