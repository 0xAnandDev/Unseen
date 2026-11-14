# UNSEEN ⌀

**Privacy-preserving image takedown demo. Your photo never leaves your device, only its fingerprint does.**

> ⚠️ This is a portfolio demo inspired by StopNCII. It is not connected to any real platform and uses test images only.

## What it does

1. You pick an image in the browser.
2. The browser computes a 256-bit perceptual hash (PDQ-style) locally. The image itself is never uploaded.
3. Only the hash is sent to the API, which stores it and searches for near-duplicates using Hamming distance.
4. When a match is registered, a simulated social platform is notified through signed webhooks (HMAC), with automatic retries on failure.

## Tech stack

| Layer | Tech |
|---|---|
| Frontend | React + TypeScript (in-browser hashing, Rust → WebAssembly planned) |
| API | Rust, Axum |
| Database | PostgreSQL (hash storage + Hamming-distance search) |
| Queue / worker | Redis-backed webhook worker with retries |
| Mock platform | Simulated social platform receiving signed webhooks |
| Infra | Docker Compose, GitHub Actions CI/CD |

## API

| Method | Path | What it does |
|---|---|---|
| `GET` | `/health` | Checks the server and database are up |
| `POST` | `/api/hashes` | Saves a hash `{ "hash": "<64 hex chars>" }`, returns `case_id` and a one-time `secret_key` |
| `POST` | `/api/hashes/search` | Finds near-duplicates `{ "hash": "...", "max_distance": 31 }` |
| `DELETE` | `/api/hashes/{case_id}` | Withdraws a case, needs header `x-secret-key` |

Hashes are compared by **Hamming distance** (how many of the 256 bits differ). Up to 31 counts as the same image by default. Only a SHA-256 of the secret key is stored. `/api/*` is rate limited per IP (burst of 10, then 2 per second) and returns `429` when exceeded.

## Run locally

Needs Docker and Rust (stable).

```bash
cp .env.example .env          # PowerShell: Copy-Item .env.example .env
docker compose up -d          # starts PostgreSQL 16
cd api
cargo run                     # API on http://localhost:8080, creates tables on start
cargo test                    # unit tests
```

## Status

🚧 Work in progress.

- [x] Week 1: Axum API + PostgreSQL in Docker, store hashes, near-match search, rate limiting
- [ ] Week 2: Mock platform + signed webhook worker with retries
- [ ] Week 3: React page with in-browser hashing
- [ ] Week 4: Deploy, tests, CI, docs

Demo only: not connected to real platforms, test images only.

## License

MIT, see [LICENSE](LICENSE).
