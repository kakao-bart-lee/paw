# CDN Setup Guide (Cloudflare R2 + API media paths)

This guide covers CDN-side configuration only. Application behavior remains unchanged.

## 1) Cloudflare R2 bucket setup

1. In Cloudflare Dashboard, create/select an **R2 bucket** for Paw media objects.
2. Configure bucket access so only trusted origins/services can list or read objects.
3. Create an R2 API token with least privilege:
   - Object Read/Write for upload workers
   - Object Read for any origin-pull integration
4. Configure the server environment to point uploads to this bucket and region.

## 2) CDN cache rule for media endpoints

Create a cache rule for media delivery endpoints:

- **Expression**: path matches `/api/v1/media/*`
- **Cache eligibility**: cache successful `GET`/`HEAD`
- **Origin cache control**: respect origin headers
- **Edge TTL**: use origin `Cache-Control` when present

Recommended origin header for immutable media responses:

`Cache-Control: public, max-age=31536000, immutable`

This enables one-year browser/edge caching for content-addressed media keys.

## 3) Origin pull configuration

When using Cloudflare as pull CDN in front of Paw API/media origin:

1. Set origin host to the Paw API domain serving media URLs.
2. Restrict origin access to Cloudflare egress ranges or mTLS (preferred).
3. Enable HTTPS-only origin pulls.
4. Forward required headers only (`Host`, auth headers when needed).
5. Do not cache authenticated/private endpoints outside `/api/v1/media/*`.

## 4) Validation checklist

- `GET` media URL responses include immutable `Cache-Control`.
- Repeated requests hit CDN cache (`CF-Cache-Status: HIT`).
- New media objects become available without purging (new key path per object).
- Private API routes remain bypassed from edge caching.
