import type { Metadata } from "next";
import "@fontsource-variable/manrope";
import "@fontsource-variable/dm-sans";
import "@fontsource/ibm-plex-mono/400.css";
import "./globals.css";
import { Sidebar } from "@/components/sidebar";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";
export const metadata: Metadata = {
  metadataBase: new URL(
    process.env.NEXT_PUBLIC_SITE_URL || "http://127.0.0.1:3000",
  ),
  title: {
    default: "Look Out API — Keep an eye on your APIs",
    template: "%s — Look Out API",
  },
  description:
    "A self-hosted Rust monitor for API health, response checks, and multi-step flows. Local history. Durable Telegram alerts. A clear signal when something changes.",
  icons: { icon: `${basePath}/icon.svg` },
  openGraph: {
    title: "Look Out API",
    description:
      "Keep an eye on your APIs. Self-hosted checks, local history, and durable incident alerts.",
    type: "website",
    images: [
      {
        url: `${basePath}/social.png`,
        width: 1200,
        height: 630,
        alt: "Look Out API — Keep an eye on your APIs",
      },
    ],
  },
  twitter: {
    images: [`${basePath}/social.png`],
    card: "summary_large_image",
    title: "Look Out API",
    description:
      "Self-hosted API monitoring, with a clear signal when something changes.",
  },
};
export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" data-scroll-behavior="smooth">
      <body>
        <a className="skip-link" href="#main">
          Skip to content
        </a>
        <Sidebar />
        <main id="main" className="main-content">
          {children}
        </main>
      </body>
    </html>
  );
}
