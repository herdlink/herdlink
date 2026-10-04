import { CommunityOverview } from "@/components/community/CommunityOverview";

export default async function CommunityPage({ params, searchParams }: { params: Promise<{ id: string }>; searchParams: Promise<{ name?: string | string[] }> }) {
  const [{ id }, { name }] = await Promise.all([params, searchParams]);
  const displayName = typeof name === "string" ? name : undefined;
  return <CommunityOverview key={`${id}:${displayName ?? ""}`} communityKey={id} displayName={displayName} />;
}
