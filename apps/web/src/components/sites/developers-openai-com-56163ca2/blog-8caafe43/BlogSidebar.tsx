import Link from "next/link";
import { articles, topics } from "./content";

const linkClasses = "block w-full rounded-[8px] py-[6px] pr-[12px] pl-[20px] text-[14px] leading-[20px] font-normal tracking-[-0.14px] text-[var(--site-text)] transition-colors duration-150 ease-[cubic-bezier(.4,0,.2,1)] hover:bg-[var(--site-hover)]";
const headingClasses = "mt-[24px] mr-0 mb-[8px] ml-[12px] text-[14px] leading-[20px] font-semibold text-[var(--site-heading)]";

export function BlogSidebar() {
  return (
    <aside className="fixed top-[64px] bottom-0 left-0 z-40 hidden w-[218px] flex-col bg-[var(--site-surface)] px-[12px] pt-[8px] pb-[24px] lg:flex">
      <nav aria-label="Blog navigation" className="min-h-0 flex-1 overflow-x-visible overflow-y-auto">
        <ul className="mt-[24px] flex list-none flex-col gap-[1px] p-0">
          <li>
            <Link
              href="/"
              aria-current="page"
              className="block w-full rounded-[8px] bg-[var(--site-hover)] px-[12px] py-[6px] text-[14px] leading-[20px] font-normal tracking-[-0.14px] text-[var(--site-text)] transition-colors duration-150 ease-[cubic-bezier(.4,0,.2,1)] hover:bg-[var(--site-hover)]"
            >
              All posts
            </Link>
          </li>
        </ul>

        <h2 className={headingClasses}>Recent</h2>
        <ul className="m-0 flex list-none flex-col gap-[1px] p-0">
          {articles.slice(0, 5).map((article) => (
            <li key={article.href}>
              <a href={article.href} className={linkClasses}>
                <span className="line-clamp-2">{article.title}</span>
              </a>
            </li>
          ))}
        </ul>

        <h2 className={headingClasses}>Topics</h2>
        <ul className="m-0 flex list-none flex-col gap-[1px] p-0">
          {topics.map((topic) => (
            <li key={topic}>
              <a
                href={`https://developers.openai.com/blog/topic/${topic.toLowerCase().replaceAll(" ", "-")}`}
                className={linkClasses}
              >
                <span className="line-clamp-2">{topic}</span>
              </a>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  );
}
