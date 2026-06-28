CREATE TYPE encryption_algo AS ENUM ('aes-256-gcm');
CREATE DOMAIN encryption_version AS SMALLINT
    CHECK (VALUE IN (1));

CREATE TABLE encrypted_vault
(
    id   SERIAL PRIMARY KEY,
    iv   VARCHAR(16)        NOT NULL, -- b64 encoded 12 bytes
    tag  VARCHAR(24)        NOT NULL, -- b64 encoded 16 bytes
    ver  encryption_version NOT NULL DEFAULT 1,
    algo encryption_algo    NOT NULL
);

CREATE TABLE encrypted_blobs
(
    id   INTEGER PRIMARY KEY REFERENCES encrypted_vault (id) ON DELETE CASCADE,
    data BYTEA NOT NULL
);

CREATE TABLE folders
(
    id             SERIAL PRIMARY KEY,
    public_id      VARCHAR(8)                                            NOT NULL UNIQUE,
    encrypted_name INT REFERENCES encrypted_blobs (id) ON DELETE CASCADE NOT NULL,
    expired_at     TIMESTAMPTZ,
    created_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP                 NOT NULL
);

CREATE TABLE files
(
    id           SERIAL PRIMARY KEY,
    public_id    VARCHAR(16)                                           NOT NULL UNIQUE,
    folder_id    INT REFERENCES folders (id) ON DELETE CASCADE         NOT NULL,

    -- encryption metadata about file, file stored in storage by `storage_path`
    data_meta_id INT REFERENCES encrypted_vault (id) ON DELETE CASCADE NOT NULL,

    -- json encoded path, mime, hash
    meta_id      INT REFERENCES encrypted_blobs (id) ON DELETE CASCADE NOT NULL,
    storage_path UUID                                                  NOT NULL UNIQUE,
    file_size    BIGINT                                                NOT NULL,
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP                 NOT NULL
);

CREATE INDEX idx_folders_public_id ON folders (public_id);
CREATE INDEX idx_files_public_id ON files (public_id);
CREATE INDEX idx_files_folder_id ON files (folder_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_blobs_id ON encrypted_blobs (id);
CREATE INDEX IF NOT EXISTS idx_files_folder_created_at ON files (folder_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_folders_expired_at ON folders (expired_at) WHERE expired_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_files_meta_lookup ON files (public_id, data_meta_id, meta_id);
