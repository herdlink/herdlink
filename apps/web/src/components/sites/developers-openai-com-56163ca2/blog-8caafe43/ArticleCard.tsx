import type { BlogArticle } from "./types";

interface ArticleCardProps {
  article: BlogArticle;
}

export function ArticleCard({ article }: ArticleCardProps) {
  return (
    <a
      href={article.href}
      className="flex w-full flex-col overflow-hidden p-0 md:px-8"
    >
      {/* The original preserves each article image's natural aspect ratio. */}
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src={article.image}
        alt={article.alt}
        width={article.imageWidth}
        height={article.imageHeight}
        className="block h-auto w-full rounded-none md:rounded-[8px]"
      />
      <div className="px-4 pt-4 text-[16px] leading-[24px] font-normal tracking-[-0.16px] text-[var(--site-secondary)] md:px-0">
        {article.date}
      </div>
      <div className="px-4 pb-2 text-[18px] leading-[24px] font-medium tracking-[-0.18px] text-[var(--site-text)] md:px-0">
        <span className="line-clamp-2">{article.title}</span>
      </div>
      <p className="m-0 line-clamp-3 px-4 text-[16px] leading-[24px] font-normal tracking-[-0.16px] text-[var(--site-text)] md:px-0">
        {article.description}
      </p>
      <div className="px-4 pt-2 text-[14px] leading-[20px] font-normal tracking-[-0.14px] text-[var(--site-secondary)] md:px-0">
        {article.topic}
      </div>
    </a>
  );
}
