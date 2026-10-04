export function BlogIntro() {
  return (
    <header className="flex flex-col items-center gap-1 pt-20 text-center">
      <h1 className="m-0 text-[30px] leading-[42px] font-semibold tracking-[-0.6px] text-[var(--site-heading)]">
        OpenAI Developer Blog
      </h1>
      <p className="m-0 max-w-xl text-[16px] leading-[26px] tracking-[-0.16px] text-[var(--site-secondary)] md:text-[18px] md:leading-[29.25px] md:tracking-[-0.18px]">
        Insights for developers building with OpenAI
      </p>
    </header>
  );
}
