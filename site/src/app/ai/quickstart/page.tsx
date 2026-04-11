export default function QuickstartPage() {
  const steps = [
    {
      num: "1",
      title: "Start the MCP Server",
      desc: "Run the Vedākṣha MCP server. It listens for JSON-RPC 2.0 connections over stdio or SSE transport.",
      code: `# Install the Vedākṣha CLI
cargo install vedaksha-cli

# Start the MCP server (stdio transport)
vedaksha mcp serve

# Or with SSE transport on a specific port
vedaksha mcp serve --transport sse --port 3001`,
      file: "terminal",
    },
    {
      num: "2",
      title: "Configure Your AI Agent's MCP Client",
      desc: "Point your AI agent's MCP client configuration to the Vedaksha server. Here is an example configuration for a typical MCP client.",
      code: `{
  "mcpServers": {
    "vedaksha": {
      "command": "vedaksha",
      "args": ["mcp", "serve"],
      "env": {
        "VEDAKSHA_LICENSE_KEY": "your-license-key"
      }
    }
  }
}`,
      file: "mcp_config.json",
    },
    {
      num: "3",
      title: "Authenticate",
      desc: "Vedākṣha MCP supports OAuth 2.1. For local development, set your license key as an environment variable. For production, use the OAuth flow.",
      code: `# Set your license key (local development)
export VEDAKSHA_LICENSE_KEY="vk_live_..."

# The MCP server reads this automatically.
# For OAuth 2.1 in production, configure the token endpoint:
# VEDAKSHA_OAUTH_TOKEN_URL=https://api.vedaksha.net/oauth/token
# VEDAKSHA_OAUTH_CLIENT_ID=your-client-id`,
      file: "terminal",
    },
    {
      num: "4",
      title: "Call compute_natal_chart",
      desc: "Your agent calls the tool with a Julian Day, latitude, longitude, ayanamsha, and house system. The response is a complete ChartGraph.",
      code: `// MCP tool call (JSON-RPC 2.0)
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "compute_natal_chart",
    "arguments": {
      "julian_day": 2460388.0,
      "latitude": 28.6139,
      "longitude": 77.2090,
      "ayanamsha": "Lahiri",
      "house_system": "WholeSign"
    }
  },
  "id": 1
}`,
      file: "request.json",
    },
    {
      num: "5",
      title: "Read the ChartGraph Response",
      desc: "The response contains a full property graph — planets, houses, signs, aspects, dignities, yogas, and ranked highlights. Each entity includes an nl_description field for natural language output.",
      code: `// Response (abbreviated)
{
  "result": {
    "chart_graph": {
      "nodes": [
        {
          "id": "planet_jupiter_40.2",
          "type": "Planet",
          "name": "Jupiter",
          "longitude": 40.2,
          "sign": "Taurus",
          "nakshatra": "Krittika",
          "house": 1,
          "dignity": "neutral",
          "nl_description": "Jupiter at 40.2° in Taurus..."
        }
      ],
      "edges": [
        {
          "from": "planet_jupiter_40.2",
          "to": "house_1",
          "type": "PLACED_IN"
        }
      ],
      "chart_highlights": [
        {
          "rank": 1,
          "feature": "Jupiter in 1st house",
          "significance": 0.92,
          "nl_description": "Jupiter in the 1st house..."
        }
      ]
    }
  }
}`,
      file: "response.json",
    },
    {
      num: "6",
      title: "Emit the Graph to Cypher",
      desc: "Convert the chart into Neo4j Cypher statements for persistent storage and cross-chart queries.",
      code: `// MCP tool call
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "emit_graph",
    "arguments": {
      "chart_graph": "<<from previous response>>",
      "format": "Cypher"
    }
  },
  "id": 2
}

// Response
{
  "result": {
    "output": "CREATE (p:Planet {id: 'planet_jupiter_40.2', name: 'Jupiter', ...})\\nCREATE (h:House {id: 'house_1', ...})\\nMERGE (p)-[:PLACED_IN]->(h)\\n..."
  }
}`,
      file: "emit_graph.json",
    },
    {
      num: "7",
      title: "Search for Transits",
      desc: "Find upcoming transit events. Results stream back as they are discovered — your agent can process them incrementally.",
      code: `// MCP tool call
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "search_transits",
    "arguments": {
      "start_jd": 2460388.0,
      "end_jd": 2460418.0,
      "bodies": ["Jupiter", "Saturn", "Mars"],
      "event_types": ["SignIngress", "Retrograde", "Conjunction"],
      "ayanamsha": "Lahiri"
    }
  },
  "id": 3
}

// Streamed response (one event at a time)
{
  "result": {
    "event": {
      "type": "SignIngress",
      "body": "Jupiter",
      "julian_day": 2460395.7,
      "sign": "Gemini",
      "nl_description": "Jupiter enters Gemini on ..."
    }
  }
}`,
      file: "transits.json",
    },
  ];

  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="px-6 pt-28 pb-16">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Quickstart
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            5-minute agent <span className="text-[#D4A843]">quickstart</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            Connect your AI agent to Vedākṣha via MCP. From zero to computing
            charts in 7 steps.
          </p>
        </div>
      </section>

      {/* Steps */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto space-y-12">
          {steps.map((step) => (
            <div key={step.num} className="scroll-mt-20">
              {/* Step header */}
              <div className="flex items-center gap-3 mb-3">
                <span className="flex items-center justify-center size-8 rounded-full bg-[#D4A843] text-white text-sm font-bold shrink-0">
                  {step.num}
                </span>
                <h2 className="text-xl font-bold uppercase tracking-wide text-[var(--color-brand-text)]">
                  {step.title}
                </h2>
              </div>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mb-4 ml-11">
                {step.desc}
              </p>

              {/* Code block */}
              <div className="ml-11 rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                  <div className="flex items-center gap-1.5">
                    <span className="size-2.5 rounded-full bg-red-400/50" />
                    <span className="size-2.5 rounded-full bg-yellow-400/50" />
                    <span className="size-2.5 rounded-full bg-green-400/50" />
                  </div>
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">
                    {step.file}
                  </span>
                </div>
                <pre className="p-4 overflow-x-auto text-sm leading-6 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
                  <code>{step.code}</code>
                </pre>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* What's next */}
      <section className="py-16 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            What&apos;s Next
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-6">
            Keep <span className="text-[#D4A843]">building</span>
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {[
              { href: "/ai/mcp-tools", title: "MCP Tool Catalog", desc: "Full documentation for all 7 tools" },
              { href: "/ai/graph", title: "Graph Output", desc: "Node types, edge types, and emit formats" },
              { href: "/ai/patterns", title: "Agent Patterns", desc: "8 real-world agent workflows" },
              { href: "/ai/comparison", title: "Why Vedākṣha", desc: "Feature comparison with alternatives" },
              { href: "/ai", title: "AI Architecture", desc: "The 10 design pillars" },
              { href: "/docs", title: "Full Documentation", desc: "Computation, Vedic, houses, and more" },
            ].map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="flex flex-col border border-[var(--color-brand-border)] rounded-lg px-4 py-3 bg-[var(--color-brand-bg)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
              >
                <span className="text-sm font-semibold text-[var(--color-brand-text)]">
                  {link.title}
                </span>
                <span className="text-xs text-[var(--color-brand-text-muted)]">
                  {link.desc}
                </span>
              </a>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
