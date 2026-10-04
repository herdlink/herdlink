import { notFound, permanentRedirect } from "next/navigation";
import { SurveyList } from "@/components/surveys/SurveyList";
import { CommunityDirectory } from "@/components/community/CommunityDirectory";

const sections = {
  surveys: { title: "Surveys", description: "Surveys will appear here." },
  institutions: { title: "Institutions", description: "Explore institutions here soon." },
  inbox: { title: "Inbox", description: "Your messages will appear here." },
  communities: { title: "Communities", description: "Find your community and join the conversation." },
};

export function generateStaticParams() {
  return Object.keys(sections).map((section) => ({ section }));
}

export default async function SectionPage({ params, searchParams }: { params: Promise<{ section: string }>; searchParams: Promise<{ community?: string }> }) {
  const { section } = await params;
  if (section === "forums") permanentRedirect("/communities");
  if (section === "surveys") { const { community } = await searchParams; return <SurveyList community={community} />; }
  if (section === "communities") return <CommunityDirectory />;
  if (!Object.hasOwn(sections, section)) notFound();
  const { title, description } = sections[section as keyof typeof sections];

  return (
    <section className="flex h-full min-h-[420px] flex-col">
      <header className="flex h-16 shrink-0 items-center border-b border-[var(--site-border)] px-6">
        <h1 className="text-[14px] font-semibold">{title}</h1>
      </header>
      <div className="flex flex-1 flex-col items-center justify-center px-6 py-16 text-center">
        <h2 className="text-2xl font-medium">{title}</h2>
        <p className="mt-3 text-sm text-[var(--site-secondary)]">{description}</p>
      </div>
    </section>
  );
}
