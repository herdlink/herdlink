"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { communityApi, errorMessage } from "@/lib/community-api";
import type { Survey } from "@/lib/survey-types";
import { buttonClass, ErrorNotice, fieldClass, secondaryClass } from "@/components/community/CommunityUi";

export function SurveyDetail({ id }: { id: string }) {
  const [survey, setSurvey] = useState<Survey | null>(null);
  const [answers, setAnswers] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [revision, setRevision] = useState(0);
  const submitting = useRef(false);
  useEffect(() => {
    let cancelled = false;
    communityApi<Survey>(`/surveys/${id}`).then((data) => { if (!cancelled) { setSurvey(data); setAnswers(data.answers ?? data.questions.map(() => "")); } }).catch((error) => { if (!cancelled) setError(errorMessage(error)); });
    return () => { cancelled = true; };
  }, [id, revision]);
  async function submit() {
    if (submitting.current || !survey) return;
    submitting.current = true; setBusy(true); setError("");
    try { setSurvey(await communityApi<Survey>(`/surveys/${id}/responses`, { answers })); setNotice("Thank you. Your response has been saved."); }
    catch (error) { setError(errorMessage(error)); }
    finally { submitting.current = false; setBusy(false); }
  }
  if (!survey) return <div className="p-8">{error ? <ErrorNotice message={error} onRetry={() => { setError(""); setRevision((v) => v + 1); }} /> : <p role="status">Loading survey…</p>}</div>;
  return <section className="mx-auto max-w-3xl p-5 md:p-8"><Link href="/surveys" className="text-sm underline">All surveys</Link><header className="mb-8 mt-6"><h1 className="text-3xl font-medium">{survey.title}</h1>{survey.description && <p className="mt-4 whitespace-pre-wrap text-sm leading-6 text-[var(--site-secondary)]">{survey.description}</p>}<div className="mt-4 flex flex-wrap gap-2">{survey.communities.map((community) => <Link key={community.id} href={`/community/${community.slug}`} className="rounded-full border border-[var(--site-border)] px-3 py-1.5 text-xs hover:bg-[var(--site-hover)]">{community.name}</Link>)}</div><p className="mt-4 text-xs text-[var(--site-secondary)]">{survey.response_count} responses · {survey.audience_count} unique community members · One response per person</p></header>
    {notice && <p role="status" className="mb-5 rounded-xl bg-[var(--site-soft)] p-4 text-sm">{notice}</p>}
    {survey.answers && <p className="mb-5 text-sm text-[var(--site-secondary)]">You have already answered this survey.</p>}
    {!survey.can_respond && <p className="mb-5 text-sm text-[var(--site-secondary)]">Join one of the selected communities to participate. Your survey is available to their members.</p>}
    <form className="space-y-6" onSubmit={(event) => { event.preventDefault(); void submit(); }}><fieldset disabled={busy || !!survey.answers || !survey.can_respond} className="space-y-6">{survey.questions.map((question, index) => <div key={question.id} className="rounded-2xl border border-[var(--site-border)] p-5"><label htmlFor={`answer-${question.id}`} className="mb-3 block text-sm font-medium">{index + 1}. {question.prompt}</label>{question.kind === "short_text" ? <textarea id={`answer-${question.id}`} className={fieldClass} rows={3} value={answers[index] ?? ""} required maxLength={4000} onChange={(event) => setAnswers((current) => current.map((a, i) => i === index ? event.target.value : a))} /> : <select id={`answer-${question.id}`} className={fieldClass} value={answers[index] ?? ""} required onChange={(event) => setAnswers((current) => current.map((a, i) => i === index ? event.target.value : a))}><option value="">Select an option</option>{question.options.map((option) => <option key={option} value={option}>{option}</option>)}</select>}</div>)}</fieldset>{error && <ErrorNotice message={error} />}{survey.can_respond && !survey.answers && <button className={buttonClass} disabled={busy || answers.some((answer) => !answer.trim())} type="submit">{busy ? "Saving…" : "Submit response"}</button>}</form>
    {survey.results && <section className="mt-10 border-t border-[var(--site-border)] pt-6"><div className="mb-5 flex items-center justify-between"><h2 className="text-xl font-medium">Survey results</h2><button className={secondaryClass} onClick={() => setRevision((v) => v + 1)} disabled={busy}>Refresh results</button></div><p className="mb-5 text-xs text-[var(--site-secondary)]">Respondent account details are not included. Written answers show the latest 50 responses.</p>{survey.questions.map((question, index) => <div key={question.id} className="mb-5 rounded-xl bg-[var(--site-soft)] p-4"><h3 className="mb-3 text-sm font-medium">{question.prompt}</h3>{question.kind === "single_choice" ? <ul className="space-y-2 text-sm">{question.options.map((option) => <li key={option} className="flex justify-between"><span>{option}</span><span>{survey.results![index].counts[option] ?? 0}</span></li>)}</ul> : survey.results![index].texts.length ? <ul className="space-y-3 text-sm">{survey.results![index].texts.map((answer, i) => <li key={i} className="whitespace-pre-wrap break-words border-b border-[var(--site-border)] pb-2">{answer}</li>)}</ul> : <p className="text-sm text-[var(--site-secondary)]">No responses yet.</p>}</div>)}</section>}
  </section>;
}
