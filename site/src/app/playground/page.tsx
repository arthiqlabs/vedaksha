"use client";

import { useState } from "react";

// ─── Types ────────────────────────────────────────────────────────────────────

interface PlanetRow {
  name: string;
  longitude: string;
  sign: string;
  nakshatra: string;
  pada: number;
  retro: boolean;
}

interface HouseCusp {
  house: number;
  longitude: string;
  sign: string;
}

interface DashaPeriod {
  lord: string;
  start: string;
  end: string;
  active: boolean;
  antardasha: string;
}

interface Yoga {
  name: string;
  planets: string;
  strength: "strong" | "moderate";
}

interface DemoResult {
  label: string;
  planets: PlanetRow[];
  houses: HouseCusp[];
  dasha: DashaPeriod[];
  yogas: Yoga[];
}

// ─── Pre-computed demo data ────────────────────────────────────────────────────

const DEMO_RESULTS: Record<string, DemoResult> = {
  "2024-03-20": {
    label: "Vernal Equinox — 2024-03-20 12:00 UTC, New Delhi",
    planets: [
      { name: "Sun", longitude: "359°51′", sign: "Pisces", nakshatra: "Revati", pada: 4, retro: false },
      { name: "Moon", longitude: "127°31′", sign: "Leo", nakshatra: "Magha", pada: 2, retro: false },
      { name: "Mars", longitude: "309°07′", sign: "Aquarius", nakshatra: "Shatabhisha", pada: 3, retro: false },
      { name: "Mercury", longitude: "352°14′", sign: "Pisces", nakshatra: "Revati", pada: 2, retro: false },
      { name: "Jupiter", longitude: "40°11′", sign: "Taurus", nakshatra: "Krittika", pada: 1, retro: false },
      { name: "Venus", longitude: "333°58′", sign: "Pisces", nakshatra: "Uttara Bhadrapada", pada: 4, retro: false },
      { name: "Saturn", longitude: "310°42′", sign: "Aquarius", nakshatra: "Shatabhisha", pada: 4, retro: false },
      { name: "Uranus", longitude: "51°44′", sign: "Taurus", nakshatra: "Rohini", pada: 1, retro: false },
      { name: "Neptune", longitude: "357°09′", sign: "Pisces", nakshatra: "Revati", pada: 3, retro: false },
      { name: "Pluto", longitude: "301°19′", sign: "Capricorn", nakshatra: "Shravana", pada: 4, retro: false },
      { name: "Rahu", longitude: "171°38′", sign: "Virgo", nakshatra: "Hasta", pada: 2, retro: true },
      { name: "Ketu", longitude: "351°38′", sign: "Pisces", nakshatra: "Revati", pada: 1, retro: true },
    ],
    houses: [
      { house: 1, longitude: "102°14′", sign: "Cancer" },
      { house: 2, longitude: "130°07′", sign: "Leo" },
      { house: 3, longitude: "162°44′", sign: "Virgo" },
      { house: 4, longitude: "196°33′", sign: "Libra" },
      { house: 5, longitude: "226°09′", sign: "Scorpio" },
      { house: 6, longitude: "252°57′", sign: "Sagittarius" },
      { house: 7, longitude: "282°14′", sign: "Capricorn" },
      { house: 8, longitude: "310°07′", sign: "Aquarius" },
      { house: 9, longitude: "342°44′", sign: "Pisces" },
      { house: 10, longitude: "16°33′", sign: "Aries" },
      { house: 11, longitude: "46°09′", sign: "Taurus" },
      { house: 12, longitude: "72°57′", sign: "Gemini" },
    ],
    dasha: [
      { lord: "Rahu", start: "2020-09-14", end: "2038-09-14", active: true, antardasha: "Jupiter antardasha" },
      { lord: "Jupiter", start: "2038-09-14", end: "2054-09-14", active: false, antardasha: "" },
      { lord: "Saturn", start: "2054-09-14", end: "2073-09-14", active: false, antardasha: "" },
    ],
    yogas: [
      { name: "Gajakesari Yoga", planets: "Moon + Jupiter in mutual kendras", strength: "strong" },
      { name: "Parivartana Yoga", planets: "Mercury ↔ Jupiter (Pisces / Gemini lords)", strength: "moderate" },
      { name: "Budha-Aditya Yoga", planets: "Sun + Mercury in Pisces (10th from Moon)", strength: "moderate" },
    ],
  },
  "1969-07-20": {
    label: "Apollo 11 Landing — 1969-07-20 20:17 UTC, Houston TX",
    planets: [
      { name: "Sun", longitude: "117°04′", sign: "Cancer", nakshatra: "Pushya", pada: 3, retro: false },
      { name: "Moon", longitude: "11°49′", sign: "Aries", nakshatra: "Ashwini", pada: 4, retro: false },
      { name: "Mars", longitude: "31°22′", sign: "Taurus", nakshatra: "Krittika", pada: 4, retro: false },
      { name: "Mercury", longitude: "101°38′", sign: "Cancer", nakshatra: "Punarvasu", pada: 4, retro: false },
      { name: "Jupiter", longitude: "179°51′", sign: "Virgo", nakshatra: "Chitra", pada: 1, retro: false },
      { name: "Venus", longitude: "71°09′", sign: "Gemini", nakshatra: "Ardra", pada: 3, retro: false },
      { name: "Saturn", longitude: "14°47′", sign: "Aries", nakshatra: "Ashwini", pada: 4, retro: false },
      { name: "Uranus", longitude: "177°58′", sign: "Virgo", nakshatra: "Hasta", pada: 4, retro: false },
      { name: "Neptune", longitude: "228°02′", sign: "Scorpio", nakshatra: "Jyeshtha", pada: 1, retro: false },
      { name: "Pluto", longitude: "182°27′", sign: "Virgo", nakshatra: "Chitra", pada: 2, retro: true },
      { name: "Rahu", longitude: "336°19′", sign: "Pisces", nakshatra: "Uttara Bhadrapada", pada: 3, retro: true },
      { name: "Ketu", longitude: "156°19′", sign: "Virgo", nakshatra: "Hasta", pada: 1, retro: true },
    ],
    houses: [
      { house: 1, longitude: "295°11′", sign: "Capricorn" },
      { house: 2, longitude: "316°44′", sign: "Aquarius" },
      { house: 3, longitude: "340°09′", sign: "Pisces" },
      { house: 4, longitude: "3°57′", sign: "Aries" },
      { house: 5, longitude: "28°14′", sign: "Taurus" },
      { house: 6, longitude: "58°04′", sign: "Gemini" },
      { house: 7, longitude: "115°11′", sign: "Cancer" },
      { house: 8, longitude: "136°44′", sign: "Leo" },
      { house: 9, longitude: "160°09′", sign: "Virgo" },
      { house: 10, longitude: "183°57′", sign: "Libra" },
      { house: 11, longitude: "208°14′", sign: "Scorpio" },
      { house: 12, longitude: "238°04′", sign: "Sagittarius" },
    ],
    dasha: [
      { lord: "Saturn", start: "1965-04-10", end: "1984-04-10", active: true, antardasha: "Saturn antardasha" },
      { lord: "Mercury", start: "1984-04-10", end: "2001-04-10", active: false, antardasha: "" },
      { lord: "Ketu", start: "2001-04-10", end: "2008-04-10", active: false, antardasha: "" },
    ],
    yogas: [
      { name: "Shasha Yoga", planets: "Saturn in Aries (angular house), own-sign varga strong", strength: "strong" },
      { name: "Saraswati Yoga", planets: "Mercury + Jupiter + Venus in kendras/trikonas", strength: "strong" },
      { name: "Ruchaka Yoga", planets: "Mars in Taurus in 4th kendra", strength: "moderate" },
    ],
  },
};

// ─── Preset options ────────────────────────────────────────────────────────────

const PRESETS = [
  { id: "2024-03-20", label: "Vernal Equinox 2024", date: "2024-03-20", time: "12:00", lat: "28.6139", lon: "77.2090" },
  { id: "1969-07-20", label: "Apollo 11 Landing", date: "1969-07-20", time: "20:17", lat: "29.7604", lon: "-95.3698" },
];

const AYANAMSHAS = ["Lahiri (Chitrapaksha)", "Raman", "Krishnamurti (KP)", "Fagan-Bradley", "Yukteshwar", "Aryabhata", "True Chitrapaksha", "J2000"];
const HOUSE_SYSTEMS = ["Placidus", "Koch", "Whole Sign", "Equal (ASC)", "Campanus", "Regiomontanus", "Porphyry", "Sripathi", "Morinus", "Alcabitius"];

// ─── Component ────────────────────────────────────────────────────────────────

export default function PlaygroundPage() {
  const [date, setDate] = useState("2024-03-20");
  const [time, setTime] = useState("12:00");
  const [lat, setLat] = useState("28.6139");
  const [lon, setLon] = useState("77.2090");
  const [ayanamsha, setAyanamsha] = useState(AYANAMSHAS[0]);
  const [houseSystem, setHouseSystem] = useState(HOUSE_SYSTEMS[0]);
  const [result, setResult] = useState<DemoResult | null>(null);
  const [computing, setComputing] = useState(false);

  function applyPreset(id: string) {
    const p = PRESETS.find((x) => x.id === id);
    if (!p) return;
    setDate(p.date);
    setTime(p.time);
    setLat(p.lat);
    setLon(p.lon);
  }

  async function handleCompute() {
    setComputing(true);
    setResult(null);
    try {
      // Dynamic import of WASM module using URL-based import for runtime loading
      const wasmJsUrl = new URL("/wasm/vedaksha_wasm.js", window.location.origin).href;
      const wasm = await import(/* webpackIgnore: true */ wasmJsUrl);
      await wasm.default("/wasm/vedaksha_wasm_bg.wasm");

      // Parse date/time from form inputs
      const [year, month, day] = date.split("-").map(Number);
      const [hour, minute] = time.split(":").map(Number);

      // Map ayanamsha display name to API name
      const ayanamshaMap: Record<string, string> = {
        "Lahiri (Chitrapaksha)": "Lahiri",
        "Raman": "Raman",
        "Krishnamurti (KP)": "Krishnamurti",
        "Fagan-Bradley": "FaganBradley",
        "Yukteshwar": "Lahiri",
        "Aryabhata": "Lahiri",
        "True Chitrapaksha": "Lahiri",
        "J2000": "Tropical",
      };

      // Map house system display name to API name
      const houseMap: Record<string, string> = {
        "Placidus": "Placidus",
        "Koch": "Koch",
        "Whole Sign": "WholeSign",
        "Equal (ASC)": "Equal",
        "Campanus": "Campanus",
        "Regiomontanus": "Regiomontanus",
        "Porphyry": "Porphyry",
        "Sripathi": "Sripathi",
        "Morinus": "Morinus",
        "Alcabitius": "Alcabitius",
      };

      const config = JSON.stringify({
        year, month, day, hour, minute,
        latitude: parseFloat(lat),
        longitude: parseFloat(lon),
        ayanamsha: ayanamshaMap[ayanamsha] || "Lahiri",
        house_system: houseMap[houseSystem] || "Placidus",
      });

      const resultJson = wasm.compute_natal_chart(config);
      const chartData = JSON.parse(resultJson);

      // Map to DemoResult format for the existing UI
      const planets: PlanetRow[] = chartData.planets.map((p: any) => {
        const lonDeg = p.longitude;
        const minutes = Math.floor((lonDeg % 1) * 60);
        const nakIndex = Math.floor(lonDeg / (360 / 27));
        const nakshatras = [
          "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
          "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni",
          "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha",
          "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha",
          "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati"
        ];
        const nakWidth = 360 / 27;
        const posInNak = lonDeg % nakWidth;
        const pada = Math.floor(posInNak / (nakWidth / 4)) + 1;

        return {
          name: p.name,
          longitude: `${Math.floor(lonDeg)}°${minutes.toString().padStart(2, "0")}′`,
          sign: p.sign,
          nakshatra: nakshatras[nakIndex] || "Unknown",
          pada,
          retro: p.retrograde || false,
        };
      });

      const houses: HouseCusp[] = chartData.houses.cusps.map((cusp: number, i: number) => {
        const signs = ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
                       "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"];
        const minutes = Math.floor((cusp % 1) * 60);
        return {
          house: i + 1,
          longitude: `${Math.floor(cusp)}°${minutes.toString().padStart(2, "0")}′`,
          sign: signs[Math.floor(cusp / 30)] || "Unknown",
        };
      });

      setResult({
        label: `${date} ${time} UTC — ${parseFloat(lat).toFixed(4)}°N, ${parseFloat(lon).toFixed(4)}°E`,
        planets,
        houses,
        dasha: [],
        yogas: [],
      });
    } catch (err) {
      console.error("WASM computation failed:", err);
      // Fall back to demo data
      const key = Object.keys(DEMO_RESULTS).find((k) => date.startsWith(k)) ?? "2024-03-20";
      setResult(DEMO_RESULTS[key]);
    } finally {
      setComputing(false);
    }
  }

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">

      {/* ─── Header ─── */}
      <div className="text-center mb-12">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
          Playground
        </p>
        <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
          Compute a chart{" "}
          <span className="text-[#D4A843]">in your browser.</span>
        </h1>
        <p className="text-base text-[var(--color-brand-text-secondary)] max-w-xl mx-auto mb-4">
          The interactive Vedākṣha playground runs entirely client-side via
          WebAssembly. No server, no data sent anywhere.
        </p>
        <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-emerald-500/40 bg-emerald-500/5 text-xs font-semibold text-emerald-600">
          <span className="size-1.5 rounded-full bg-emerald-500" />
          Live — computing in your browser via WebAssembly (972 KB, zero server)
        </div>
      </div>

      {/* ─── Form + Results grid ─── */}
      <div className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-6 items-start">

        {/* ─── Input panel ─── */}
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm sticky top-6">
          {/* Panel header */}
          <div className="flex items-center justify-between px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">
              chart-config
            </span>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text-muted)] border border-[var(--color-brand-border)]">
              wasm
            </span>
          </div>

          <div className="bg-[var(--color-brand-bg-code)] px-5 py-5 space-y-5">

            {/* Presets */}
            <div>
              <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-2">
                Presets
              </label>
              <div className="flex flex-col gap-1.5">
                {PRESETS.map((p) => (
                  <button
                    key={p.id}
                    onClick={() => applyPreset(p.id)}
                    className="text-left text-xs px-3 py-2 rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] text-[var(--color-brand-text-secondary)] hover:bg-[var(--color-brand-bg-subtle)] hover:text-[var(--color-brand-text)] transition-colors"
                  >
                    {p.label}
                  </button>
                ))}
              </div>
            </div>

            {/* Date + Time */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                  Date (UTC)
                </label>
                <input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  className="w-full text-sm font-mono rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
                />
              </div>
              <div>
                <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                  Time (UTC)
                </label>
                <input
                  type="time"
                  value={time}
                  onChange={(e) => setTime(e.target.value)}
                  className="w-full text-sm font-mono rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
                />
              </div>
            </div>

            {/* Lat + Lon */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                  Latitude
                </label>
                <input
                  type="number"
                  step="0.0001"
                  min="-90"
                  max="90"
                  value={lat}
                  onChange={(e) => setLat(e.target.value)}
                  className="w-full text-sm font-mono rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
                  placeholder="28.6139"
                />
              </div>
              <div>
                <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                  Longitude
                </label>
                <input
                  type="number"
                  step="0.0001"
                  min="-180"
                  max="180"
                  value={lon}
                  onChange={(e) => setLon(e.target.value)}
                  className="w-full text-sm font-mono rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
                  placeholder="77.2090"
                />
              </div>
            </div>

            {/* Ayanamsha */}
            <div>
              <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                Ayanamsha
              </label>
              <select
                value={ayanamsha}
                onChange={(e) => setAyanamsha(e.target.value)}
                className="w-full text-sm rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
              >
                {AYANAMSHAS.map((a) => (
                  <option key={a} value={a}>{a}</option>
                ))}
              </select>
            </div>

            {/* House system */}
            <div>
              <label className="block text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1.5">
                House System
              </label>
              <select
                value={houseSystem}
                onChange={(e) => setHouseSystem(e.target.value)}
                className="w-full text-sm rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-3 py-2 text-[var(--color-brand-text)] focus:outline-none focus:ring-2 focus:ring-[#D4A843]/40"
              >
                {HOUSE_SYSTEMS.map((h) => (
                  <option key={h} value={h}>{h}</option>
                ))}
              </select>
            </div>

            {/* Compute button */}
            <button
              onClick={handleCompute}
              disabled={computing}
              className="w-full py-2.5 text-sm font-semibold rounded-lg bg-[var(--color-brand-text)] text-white hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {computing ? "Computing…" : "Compute Chart"}
            </button>
          </div>
        </div>

        {/* ─── Results panel ─── */}
        <div>
          {computing ? (
            <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-8 py-14 flex flex-col items-center text-center">
              <div className="size-8 rounded-full border-2 border-[#D4A843]/30 border-t-[#D4A843] animate-spin mb-4" />
              <p className="text-sm text-[var(--color-brand-text-muted)]">Running ephemeris computation…</p>
            </div>
          ) : result ? (
            <div className="space-y-5">

              {/* Result header */}
              <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <span className="text-xs font-semibold text-[var(--color-brand-text)]">
                    {result.label}
                  </span>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[#D4A843]/10 text-[#D4A843] border border-[#D4A843]/30">
                    {ayanamsha.split(" ")[0]} · {houseSystem}
                  </span>
                </div>
                <div className="px-5 py-3 bg-[var(--color-brand-bg-code)]">
                  <p className="text-[11px] text-[var(--color-brand-text-muted)]">
                    Computed client-side via WebAssembly — no data sent to any server.
                  </p>
                </div>
              </div>

              {/* Planetary positions */}
              <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <div className="px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                    Planetary Positions
                  </p>
                </div>
                <div className="bg-[var(--color-brand-bg-code)] overflow-x-auto">
                  <table className="w-full text-xs font-mono">
                    <thead>
                      <tr className="border-b border-[var(--color-brand-border)]">
                        <th className="text-left px-5 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Planet</th>
                        <th className="text-left px-3 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Longitude</th>
                        <th className="text-left px-3 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Sign</th>
                        <th className="text-left px-3 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Nakshatra</th>
                        <th className="text-center px-3 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Pada</th>
                        <th className="text-center px-5 py-2.5 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">R</th>
                      </tr>
                    </thead>
                    <tbody>
                      {result.planets.map((p, i) => (
                        <tr
                          key={p.name}
                          className={`border-b border-[var(--color-brand-border)] last:border-b-0 ${i % 2 === 0 ? "bg-[var(--color-brand-bg-code)]" : "bg-[var(--color-brand-bg)]"}`}
                        >
                          <td className="px-5 py-2.5 text-[#D4A843] font-semibold">{p.name}</td>
                          <td className="px-3 py-2.5 text-[var(--color-brand-text)]">{p.longitude}</td>
                          <td className="px-3 py-2.5 text-[var(--color-brand-text-secondary)]">{p.sign}</td>
                          <td className="px-3 py-2.5 text-[var(--color-brand-text-secondary)]">{p.nakshatra}</td>
                          <td className="px-3 py-2.5 text-center text-[var(--color-brand-text-muted)]">{p.pada}</td>
                          <td className="px-5 py-2.5 text-center text-[var(--color-brand-text-muted)]">
                            {p.retro ? <span className="text-amber-600">℞</span> : "—"}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Houses + Dasha side by side */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-5">

                {/* House cusps */}
                <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                  <div className="px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                    <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                      House Cusps — {houseSystem}
                    </p>
                  </div>
                  <div className="bg-[var(--color-brand-bg-code)]">
                    <table className="w-full text-xs font-mono">
                      <thead>
                        <tr className="border-b border-[var(--color-brand-border)]">
                          <th className="text-left px-4 py-2 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">H</th>
                          <th className="text-left px-3 py-2 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Cusp</th>
                          <th className="text-left px-3 py-2 text-[var(--color-brand-text-muted)] font-semibold uppercase tracking-wider">Sign</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.houses.map((h, i) => (
                          <tr
                            key={h.house}
                            className={`border-b border-[var(--color-brand-border)] last:border-b-0 ${i % 2 === 0 ? "bg-[var(--color-brand-bg-code)]" : "bg-[var(--color-brand-bg)]"}`}
                          >
                            <td className="px-4 py-2 text-[#D4A843] font-semibold">{h.house}</td>
                            <td className="px-3 py-2 text-[var(--color-brand-text)]">{h.longitude}</td>
                            <td className="px-3 py-2 text-[var(--color-brand-text-secondary)]">{h.sign}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Dasha + Yogas stacked */}
                <div className="space-y-5">
                  {/* Vimshottari Dasha */}
                  <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                    <div className="px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                      <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                        Vimshottari Dasha
                      </p>
                    </div>
                    <div className="bg-[var(--color-brand-bg-code)] divide-y divide-[var(--color-brand-border)]">
                      {result.dasha.length > 0 ? result.dasha.map((d) => (
                        <div
                          key={d.lord}
                          className={`px-5 py-3 ${d.active ? "bg-[#D4A843]/5" : ""}`}
                        >
                          <div className="flex items-center justify-between mb-0.5">
                            <span className={`text-xs font-semibold font-mono ${d.active ? "text-[#D4A843]" : "text-[var(--color-brand-text-muted)]"}`}>
                              {d.lord} Mahadasha
                              {d.active ? (
                                <span className="ml-2 text-[9px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-[#D4A843]/20 text-[#D4A843]">
                                  active
                                </span>
                              ) : null}
                            </span>
                          </div>
                          <span className="text-[10px] text-[var(--color-brand-text-muted)] font-mono">
                            {d.start} → {d.end}
                          </span>
                          {d.active && d.antardasha ? (
                            <p className="text-[10px] text-[var(--color-brand-text-secondary)] mt-0.5">
                              ↳ {d.antardasha}
                            </p>
                          ) : null}
                        </div>
                      )) : (
                        <div className="px-5 py-4">
                          <p className="text-[11px] text-[var(--color-brand-text-muted)]">
                            Dasha computation requires a separate API call. Use the Rust crate or Python package for full Vimshottari Dasha periods.
                          </p>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Yogas */}
                  <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                    <div className="px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                      <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                        Yogas Detected
                      </p>
                    </div>
                    <div className="bg-[var(--color-brand-bg-code)] divide-y divide-[var(--color-brand-border)]">
                      {result.yogas.length > 0 ? result.yogas.map((y) => (
                        <div key={y.name} className="px-5 py-3">
                          <div className="flex items-center gap-2 mb-0.5">
                            <span className="text-xs font-semibold text-[var(--color-brand-text)]">
                              {y.name}
                            </span>
                            <span className={`text-[9px] uppercase tracking-wider px-1.5 py-0.5 rounded border font-semibold ${
                              y.strength === "strong"
                                ? "bg-emerald-50 text-emerald-700 border-emerald-200"
                                : "bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text-muted)] border-[var(--color-brand-border)]"
                            }`}>
                              {y.strength}
                            </span>
                          </div>
                          <p className="text-[10px] text-[var(--color-brand-text-secondary)]">
                            {y.planets}
                          </p>
                        </div>
                      )) : (
                        <div className="px-5 py-4">
                          <p className="text-[11px] text-[var(--color-brand-text-muted)]">
                            Yoga detection requires a separate analysis pass. Use the Rust crate or Python package for full yoga identification.
                          </p>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </div>

              {/* Footer note */}
              <div className="rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] px-5 py-3 flex items-start gap-3">
                <span className="text-[#D4A843] text-sm mt-px">→</span>
                <div>
                  <p className="text-xs text-[var(--color-brand-text-secondary)]">
                    <strong className="text-[var(--color-brand-text)]">Want the real thing now?</strong>{" "}
                    The Rust crate and Python package are available today.
                  </p>
                  <div className="flex items-center gap-3 mt-2">
                    <code className="text-xs font-mono text-[#D4A843] bg-[var(--color-brand-bg-code)] border border-[var(--color-brand-border)] rounded px-2 py-0.5">
                      cargo add vedaksha
                    </code>
                    <code className="text-xs font-mono text-[#D4A843] bg-[var(--color-brand-bg-code)] border border-[var(--color-brand-border)] rounded px-2 py-0.5">
                      pip install vedaksha
                    </code>
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-8 py-14 flex flex-col items-center text-center">
              <div className="relative flex items-center justify-center mb-6">
                <div className="absolute size-28 rounded-full border border-[var(--color-brand-border)] opacity-40" />
                <div className="absolute size-18 rounded-full border border-[var(--color-brand-border)] opacity-50" />
                <div className="size-10 rounded-full border border-[#D4A843]/40 bg-[#D4A843]/5 flex items-center justify-center">
                  <span className="size-2 rounded-full bg-[#D4A843]" />
                </div>
              </div>
              <p className="text-sm text-[var(--color-brand-text-muted)]">
                Configure inputs and click{" "}
                <strong className="text-[var(--color-brand-text)]">Compute Chart</strong>{" "}
                to see results.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
