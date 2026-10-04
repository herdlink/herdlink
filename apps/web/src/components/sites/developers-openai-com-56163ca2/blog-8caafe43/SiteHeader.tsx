import { ExternalLinkIcon, MenuIcon, MoonIcon, SearchIcon, SunIcon } from "../shared/icons";
import { NavigationMenu } from "./NavigationMenu";
import { navigationMenus } from "./navigation";

interface SiteHeaderProps {
  theme: string;
  menuOpen: boolean;
  onSearch: () => void;
  onToggleTheme: () => void;
  onToggleMenu: () => void;
}

export function SiteHeader({ theme, menuOpen, onSearch, onToggleTheme, onToggleMenu }: SiteHeaderProps) {
  return (
    <header className="site-header fixed top-0 z-50 h-16 border-b border-[var(--site-border)] bg-[var(--site-surface)]">
      <div className="flex h-full items-center px-4 md:px-8 lg:grid lg:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] lg:gap-6">
        <a href="https://developers.openai.com/" className="order-1 flex min-h-11 shrink-0 items-center lg:col-start-1 lg:row-start-1 lg:-ml-2 lg:justify-self-start">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img className="site-logo h-6 w-48" src="/sites/developers-openai-com-56163ca2/shared/icons/logo.svg" alt="OpenAI Developers" width="192" height="24" />
        </a>
        <div className="order-2 ml-auto flex shrink-0 items-center gap-4 md:gap-3 lg:col-start-3 lg:row-start-1 lg:ml-0 lg:justify-self-end lg:gap-5">
          <button onClick={onSearch} className="hidden min-w-52 items-center justify-between gap-3 rounded-full border border-[var(--site-border)] bg-[var(--site-panel)] px-4 py-2 text-[14px] leading-5 text-[var(--site-secondary)] transition-colors hover:bg-[var(--site-hover)] 2xl:flex">
            Start searching<SearchIcon className="site-icon" />
          </button>
          <a href="https://platform.openai.com/login" target="_blank" rel="noopener noreferrer" className="hidden items-center gap-2 rounded-full bg-[var(--site-solid)] px-4 py-2 text-[14px] leading-5 font-medium text-[var(--site-solid-text)] transition-opacity hover:opacity-85 lg:flex">
            API Dashboard<ExternalLinkIcon className="h-3.5 w-3.5" />
          </a>
          <button aria-label="Toggle light and dark theme" onClick={onToggleTheme} className="hidden text-[var(--site-secondary)] transition-colors hover:text-[var(--site-text)] lg:flex">
            {theme === "dark" ? <MoonIcon className="site-icon" /> : <SunIcon className="site-icon" />}
          </button>
          <button aria-label="Search developer resources" onClick={onSearch} className="flex h-11 w-11 items-center justify-center rounded-full text-[var(--site-secondary)] transition-colors hover:bg-[var(--site-hover)] hover:text-[var(--site-text)] 2xl:hidden">
            <SearchIcon className="site-icon" />
          </button>
          <button aria-label="Toggle menu" aria-expanded={menuOpen} aria-controls="mobile-navigation" onClick={onToggleMenu} className={`relative right-1 flex h-11 w-11 items-center justify-center rounded-full text-[var(--site-secondary)] transition-colors hover:bg-[var(--site-hover)] md:right-0 lg:hidden`}>
            <MenuIcon className="site-icon" />
          </button>
        </div>
        <nav aria-label="Primary navigation" className="order-3 hidden items-center gap-1 lg:col-start-2 lg:row-start-1 lg:flex">
          <NavigationMenu label="Home" href="https://developers.openai.com/" />
          <NavigationMenu label="API" href="https://developers.openai.com/api/docs" items={navigationMenus.API} />
          <NavigationMenu label="ChatGPT" href="https://developers.openai.com/chatgpt" items={navigationMenus.ChatGPT} />
          <NavigationMenu label="Resources" href="https://developers.openai.com/learn" items={navigationMenus.Resources} active />
        </nav>
      </div>
    </header>
  );
}
