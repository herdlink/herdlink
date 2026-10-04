# Herdlink frontend

Next.js, React, Tailwind CSS and Sigma.js. See the [project README](../../README.md)
for features and local setup.

From the repository root:

```sh
just frontend-install
just frontend       # http://localhost:3001
just frontend-check
just frontend-build
```

The frontend proxies `/api` to `http://127.0.0.1:3000`. Set `BACKEND_URL` in
`.env.local` to use another backend, then restart the frontend. The login form
prefills `demo@herdlink.local` / `HerdlinkDemo123!`; the demo has the ordinary user role.

Use Bun and the committed `bun.lock` for reproducible frontend installs.
