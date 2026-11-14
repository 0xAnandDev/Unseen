-- One row per reported image fingerprint.
-- We never store the image itself, only its 256-bit perceptual hash.
CREATE TABLE IF NOT EXISTS hashes (
    id          BIGSERIAL PRIMARY KEY,
    case_id     UUID        NOT NULL UNIQUE,       -- public ID we give back to the user
    hash_hex    CHAR(64)    NOT NULL,              -- the hash as 64 hex characters (easy to read)
    hash_bits   BIT(256)    NOT NULL,              -- the same hash as raw bits (fast to compare)
    secret_hash TEXT        NOT NULL,              -- SHA-256 of the user's secret key, never the key itself
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
