import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Herdlink",
  description: "Explore your graph, conversations, and community with Herdlink.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased">
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}
