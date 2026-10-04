import { CommunityOverview } from "@/components/community/CommunityOverview";

export default async function CommunityPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <CommunityOverview key={id} communityKey={id} />;
}
