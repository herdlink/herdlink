import { ChevronDownIcon } from "../shared/icons";
import type { NavigationItem } from "./navigation";
import styles from "./NavigationMenu.module.css";

interface NavigationMenuProps {
  label: string;
  href: string;
  items?: NavigationItem[];
  active?: boolean;
}

function localHref(href: string) {
  return href === "https://developers.openai.com/blog" ? "/" : href;
}

export function NavigationMenu({
  label,
  href,
  items,
  active = false,
}: NavigationMenuProps) {
  const hasMenu = Boolean(items?.length);

  return (
    <div className={styles.wrapper}>
      <a
        href={localHref(href)}
        className={styles.topLink}
        data-active={active || undefined}
      >
        {label}
        {hasMenu && <ChevronDownIcon className={styles.chevron} />}
      </a>
      {hasMenu && (
        <div className={styles.dropdown}>
          <div className={styles.menu}>
            {items?.map((item) => (
              <a
                key={item.href}
                href={localHref(item.href)}
                className={styles.item}
              >
                <span className={styles.itemContent}>
                  <span className={styles.itemLabel}>{item.label}</span>
                  <span className={styles.description}>{item.description}</span>
                </span>
              </a>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
