export default function Home() {
  return (
    <section aria-labelledby="graph-heading" className="flex h-full min-h-[420px] flex-col">
      <header className="flex h-16 shrink-0 items-center border-b border-[var(--site-border)] px-6">
        <h1 id="graph-heading" className="text-[14px] font-semibold">Graph view</h1>
      </header>
      <div aria-label="Graph visualization" className="flex-1" />
    </section>
  );
}
