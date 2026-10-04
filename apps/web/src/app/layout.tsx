import type { Metadata } from "next";
import localFont from "next/font/local";
import "./globals.css";

const openaiSans = localFont({
  src: [
    { path: "../../public/sites/developers-openai-com-56163ca2/shared/fonts/OpenAISans-Regular.woff2", weight: "400", style: "normal" },
    { path: "../../public/sites/developers-openai-com-56163ca2/shared/fonts/OpenAISans-Medium.woff2", weight: "500", style: "normal" },
    { path: "../../public/sites/developers-openai-com-56163ca2/shared/fonts/OpenAISans-Semibold.woff2", weight: "600", style: "normal" },
  ],
  variable: "--font-openai-sans",
  display: "swap",
});

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
    <html
      lang="en"
      className={`${openaiSans.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}
