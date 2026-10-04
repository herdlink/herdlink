export const fieldClass = "w-full rounded-xl border border-[var(--site-border)] bg-[var(--site-soft)] px-4 py-3 text-sm outline-none focus:ring-1 focus:ring-[var(--site-secondary)] disabled:opacity-50";
export const buttonClass = "inline-flex items-center justify-center gap-2 rounded-full bg-[var(--site-solid)] px-4 py-2 text-sm font-medium text-[var(--site-solid-text)] disabled:cursor-not-allowed disabled:opacity-40";
export const secondaryClass = "inline-flex items-center justify-center gap-2 rounded-full border border-[var(--site-border)] px-4 py-2 text-sm hover:bg-[var(--site-hover)] disabled:opacity-40";

export function ErrorNotice({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div role="alert" className="rounded-xl border border-[var(--site-border)] bg-[var(--site-soft)] p-4 text-sm">
      <p>{message}</p>
      {onRetry && <button type="button" onClick={onRetry} className={`${secondaryClass} mt-3`}>Try again</button>}
    </div>
  );
}

export function Author({ authorId, userId, createdAt }: { authorId: string; userId: string; createdAt: string }) {
  return (
    <div className="flex flex-wrap items-center gap-2 text-xs text-[var(--site-secondary)]">
      <span className="font-medium text-[var(--site-text)]">{authorId === userId ? "You" : `Member ${authorId.slice(0, 8)}`}</span>
      <time dateTime={createdAt}>{new Date(createdAt).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}</time>
    </div>
  );
}
