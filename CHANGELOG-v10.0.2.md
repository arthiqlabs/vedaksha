# v10.0.2

Patch release. No engine change: every computed value is identical to
v10.0.1. This release fixes how the MCP server answers notifications.

## Fixed

- The MCP server answered JSON-RPC notifications with a parse error.
  Every MCP client sends `notifications/initialized` right after
  `initialize`, and that message has no `id`, so the server replied with
  error -32700 ("Parse error: missing field `id`") on every connection.
  JSON-RPC 2.0 forbids answering a
  notification. A notification now gets no reply over stdio, and
  `202 Accepted` with an empty body over HTTP, as the MCP HTTP transport
  specifies. The Python MCP server (`vedaksha mcp`) had the same defect and
  is fixed the same way. A request with an explicit `"id": null` is still a
  request and is still answered.

  Tools kept working after the error, which is why clients that ignore
  unexpected messages never noticed. A stricter client could reject the
  connection or log an error each time.

## Compatibility

Identical computed values to v10.0.1. `McpServer::handle_request` is
unchanged; the new `McpServer::handle_message` is what a transport should
call, because it also handles notifications.
