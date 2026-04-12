import type { Metadata } from "next";
import { Inter, JetBrains_Mono, Noto_Sans_Devanagari } from "next/font/google";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700"],
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-jetbrains-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
});

const notoSansDevanagari = Noto_Sans_Devanagari({
  variable: "--font-noto-sans-devanagari",
  subsets: ["devanagari"],
  weight: ["400", "600"],
});

const SITE_URL = "https://vedaksha.net";
const DESCRIPTION =
  "The astronomical ephemeris for the agentic age. Clean-room Rust implementation with sub-arcsecond planetary precision.";

export const metadata: Metadata = {
  title: "Vedākṣha — Axis of Wisdom",
  description: DESCRIPTION,
  metadataBase: new URL(SITE_URL),
  icons: {
    icon: "/logo/logo-favicon.svg",
  },
  openGraph: {
    type: "website",
    url: SITE_URL,
    siteName: "Vedākṣha",
    title: "Vedākṣha — Axis of Wisdom",
    description: DESCRIPTION,
    locale: "en_US",
  },
  twitter: {
    card: "summary",
    title: "Vedākṣha — Axis of Wisdom",
    description: DESCRIPTION,
    site: "@vedaksha",
  },
  alternates: {
    canonical: SITE_URL,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${inter.variable} ${jetbrainsMono.variable} ${notoSansDevanagari.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-[var(--color-brand-bg)] text-[var(--color-brand-text)]">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Vedākṣha",
              "alternateName": "Vedaksha",
              "description": "Astronomical ephemeris and Vedic astrology computation platform. Clean-room Rust implementation with sub-arcsecond precision.",
              "url": "https://vedaksha.net",
              "applicationCategory": "DeveloperApplication",
              "operatingSystem": "Cross-platform",
              "programmingLanguage": ["Rust", "Python", "WebAssembly"],
              "offers": {
                "@type": "Offer",
                "price": "500",
                "priceCurrency": "USD",
                "description": "Commercial license — one-time per organization"
              },
              "author": {
                "@type": "Organization",
                "name": "ArthIQ Labs LLC",
                "url": "https://vedaksha.net",
                "email": "info@arthiq.net"
              },
              "license": "https://vedaksha.net/legal/bsl"
            })
          }}
        />
        <Header />
        <main className="flex-1">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
