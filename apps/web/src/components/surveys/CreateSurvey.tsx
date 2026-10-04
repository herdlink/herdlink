"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { ClipboardPlus, Plus, Trash2, X } from "lucide-react";
import { communityApi, errorMessage } from "@/lib/community-api";
import type { Survey, SurveyQuestion, SurveyTarget } from "@/lib/survey-types";
import { buttonClass, ErrorNotice, fieldClass, secondaryClass } from "@/components/community/CommunityUi";

export function CreateSurveyButton({ targets }: { targets: SurveyTarget[] }) {
  const [open, setOpen] = useState(false);
  return <><button type="button" disabled={!targets.length} title={targets.length ? "Survey members of disease communities in this graph" : "Add a disease to the graph first"} onClick={() => setOpen(true)} className={`${buttonClass} text-xs`}><ClipboardPlus size={14} />Create Survey</button>{open && <CreateSurvey targets={targets} onClose={() => setOpen(false)} />}</>;
}
function CreateSurvey({ targets, onClose }: { targets: SurveyTarget[]; onClose: () => void }) {
  const router = useRouter();
  const dialog = useRef<HTMLDialogElement>(null);
  const submitting = useRef(false);
  const [selected, setSelected] = useState(targets.map((target) => target.key));
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [questions, setQuestions] = useState<SurveyQuestion[]>([{ id: "question-1", prompt: "", kind: "short_text", options: [] }]);
  const [audience, setAudience] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const selectedTargets = targets.filter((target) => selected.includes(target.key));
  const targetJson = JSON.stringify(selectedTargets);
  useEffect(() => { dialog.current?.showModal(); const element = dialog.current; return () => element?.close(); }, []);
  useEffect(() => {
    let cancelled = false;
    if (JSON.parse(targetJson).length) communityApi<{ audience_count: number }>("/surveys/audience", { communities: JSON.parse(targetJson) }).then((data) => { if (!cancelled) setAudience(data.audience_count); }).catch(() => { if (!cancelled) setAudience(null); });
    return () => { cancelled = true; };
  }, [targetJson]);
  function changeQuestion(index: number, changes: Partial<SurveyQuestion>) { setQuestions((current) => current.map((question, i) => i === index ? { ...question, ...changes } : question)); }
  const valid = title.trim() && selected.length && questions.every((q) => q.prompt.trim() && (q.kind === "short_text" || (q.options.length >= 2 && q.options.every((o) => o.trim()) && new Set(q.options.map((o) => o.trim())).size === q.options.length)));
  async function publish() {
    if (!valid || submitting.current) return;
    submitting.current = true; setBusy(true); setError("");
    try {
      const survey = await communityApi<Survey>("/surveys", { title: title.trim(), description: description.trim(), questions, communities: selectedTargets });
      onClose(); router.push(`/surveys/${survey.id}`);
    } catch (error) { setError(errorMessage(error)); }
    finally { submitting.current = false; setBusy(false); }
  }
  return <dialog ref={dialog} aria-labelledby="create-survey-heading" onCancel={(event) => { if (busy) event.preventDefault(); else onClose(); }} className="m-auto max-h-[90dvh] w-[calc(100%-2rem)] max-w-2xl overflow-y-auto rounded-3xl border border-[var(--site-border)] bg-[var(--site-panel)] p-6 text-[var(--site-text)] shadow-xl backdrop:bg-black/40">
    <div className="mb-5 flex items-center justify-between"><h2 id="create-survey-heading" className="text-xl font-medium">Create Survey</h2><button type="button" disabled={busy} aria-label="Close survey creator" onClick={onClose} className="rounded-full p-2 hover:bg-[var(--site-hover)]"><X size={18} /></button></div>
    <form onSubmit={(event) => { event.preventDefault(); void publish(); }} className="space-y-5">
      <label className="block text-sm font-medium">Survey title<input className={`${fieldClass} mt-2`} value={title} onChange={(event) => setTitle(event.target.value)} maxLength={200} required autoFocus disabled={busy} placeholder="What would you like to learn?" /></label>
      <label className="block text-sm font-medium">Description (optional)<textarea className={`${fieldClass} mt-2`} value={description} onChange={(event) => setDescription(event.target.value)} maxLength={3000} rows={2} disabled={busy} placeholder="Tell participants what this survey is about." /></label>
      <fieldset disabled={busy} className="rounded-xl border border-[var(--site-border)] p-4"><legend className="px-1 text-sm font-medium">Disease communities from your graph</legend><div className="space-y-2">{targets.map((target) => <label key={target.key} className="flex items-center gap-2 text-sm"><input type="checkbox" checked={selected.includes(target.key)} onChange={() => setSelected((current) => current.includes(target.key) ? current.filter((key) => key !== target.key) : [...current, target.key])} />{target.name}</label>)}</div><p className="mt-3 text-xs leading-5 text-[var(--site-secondary)]">{selected.length ? `${selected.length} communities · ${audience === null ? "Counting members…" : `${audience} unique members`}` : "Select at least one community."} Each person can answer once. People who join these communities later can participate too.</p></fieldset>
      <div className="space-y-4">{questions.map((question, index) => <fieldset key={question.id} disabled={busy} className="space-y-3 rounded-xl border border-[var(--site-border)] p-4"><div className="flex justify-between"><p className="text-sm font-medium">Question {index + 1}</p>{questions.length > 1 && <button aria-label={`Remove question ${index + 1}`} type="button" onClick={() => setQuestions((current) => current.filter((q) => q.id !== question.id))} className="text-[var(--site-secondary)]"><Trash2 size={14} /></button>}</div><label className="block text-sm">Question<input className={`${fieldClass} mt-2`} value={question.prompt} onChange={(event) => changeQuestion(index, { prompt: event.target.value })} required maxLength={500} placeholder="Write your question…" /></label><div><label htmlFor={`answer-type-${question.id}`} className="block text-sm">Answer type</label><select id={`answer-type-${question.id}`} className={`${fieldClass} mt-2`} value={question.kind} onChange={(event) => changeQuestion(index, { kind: event.target.value as SurveyQuestion["kind"], options: event.target.value === "single_choice" ? ["", ""] : [] })}><option value="short_text">Written answer</option><option value="single_choice">Choose one option</option></select></div>{question.kind === "single_choice" && <div className="space-y-2">{question.options.map((option, i) => <label key={i} className="block text-xs">Option {i + 1}<input className={`${fieldClass} mt-1`} value={option} onChange={(event) => changeQuestion(index, { options: question.options.map((o, j) => j === i ? event.target.value : o) })} required maxLength={150} /></label>)}{question.options.length < 8 && <button type="button" className={secondaryClass} onClick={() => changeQuestion(index, { options: [...question.options, ""] })}>Add option</button>}</div>}</fieldset>)}</div>
      {questions.length < 10 && <button type="button" disabled={busy} className={secondaryClass} onClick={() => setQuestions((current) => [...current, { id: crypto.randomUUID(), prompt: "", kind: "short_text", options: [] }])}><Plus size={14} />Add question</button>}
      {error && <ErrorNotice message={error} />}
      <div className="flex justify-end gap-3"><button type="button" disabled={busy} className={secondaryClass} onClick={onClose}>Cancel</button><button type="submit" disabled={!valid || busy} className={buttonClass}>{busy ? "Publishing…" : "Publish survey"}</button></div>
    </form>
  </dialog>;
}
