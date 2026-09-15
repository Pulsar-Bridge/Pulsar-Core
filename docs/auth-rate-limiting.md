# Auth rate limiting

`admin_auth` and `api_key_auth` (`src/middleware/auth.rs`) are on the request
path before any handler runs, so they're the trust boundary a
credential-stuffing or key-guessing run would hit first. Both are rate
limited the same way: