# Vedaksha v7.3.1

**A release whose only job is to make the container listable.** v7.3.0 published to crates.io, npm,
PyPI, GHCR and the GitHub Release, and then the MCP Registry rejected it three times. Two of those
were file edits. The third needed a new image, and a new image needs a new tag.

No code changes. No computed value moves. The Rust source is byte-identical to v7.3.0 — only the
Dockerfile, `server.json`, a new guard and the workflows differ.

---

## What the registry actually requires

Each of these is enforced by `registry.modelcontextprotocol.io` at publish time and **absent from
the JSON Schema it publishes**, which v7.3.0's package block validated against cleanly:

1. an OCI package must **not** carry `registryBaseUrl` — the registry wants a canonical reference
   in `identifier`, tag included;
2. an OCI package must **not** carry `version` — which that same schema marks as **required**;
3. the image must carry
   `LABEL io.modelcontextprotocol.server.name="io.github.arthiqlabs/vedaksha"`, which is how the
   registry proves the publisher owns the image.

Schema-valid is not the same as accepted. Worse, all three are checked *after* every other publish
has succeeded, so there is no way to learn them except by being rejected — one at a time — with the
irreversible work already done.

## Why this needed a release rather than a fix

The label lives in the image. The v7.3.0 image was built from a Dockerfile that did not carry it,
and rebuilding that tag would put an artifact behind `v7.3.0` which the tagged source never
produced. That is precisely the provenance this project exists to defend, so the v7.3.0 image
stands unchanged and **v7.3.0 will never appear in the MCP Registry**.

v7.3.1 is the first tag whose image carries the label, which makes it the first that can be listed
with an installable package:

```
docker run --rm -i ghcr.io/arthiqlabs/vedaksha-mcp:v7.3.1 --stdio
```

## The guard that would have caught it in three seconds

`scripts/check_mcp_image_label.py` runs in `make gate`, `make guards`, `ci.yml` and `release.yml`'s
guard step. It asserts the Dockerfile label exists and equals `server.json`'s `name`, that no OCI
package carries `registryBaseUrl` or `version`, and that the identifier is a canonical tagged
reference. It was red-proven four ways — wrong label, missing label, restored `version`, untagged
identifier — before being wired in.

`publish-mcp-registry.yml` is a new dispatch-only workflow that publishes the committed
`server.json` from the default branch. Re-running a failed registry job cannot help, because it
checks out the tag whose file was rejected, and this project does not move published tags.

## Ordering

`publish-mcp-registry` now also depends on `docker`. It previously needed only
`[version-check, test]`, so from the moment `server.json` began advertising the image as an
installable package, the registry entry could have been published before the image with that tag
existed. That was corrected before v7.3.0 shipped and is recorded here for completeness.
