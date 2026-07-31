CREATE TYPE encryption_algo AS ENUM ('aes-256-gcm');
CREATE DOMAIN encryption_version AS SMALLINT
    CHECK (VALUE IN (1));

CREATE TABLE folders
(
    id             BIGSERIAL PRIMARY KEY,
    public_id      VARCHAR(8)                            NOT NULL UNIQUE,
    encrypted_name JSONB                                 NOT NULL,
    expired_at     TIMESTAMPTZ,
    created_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE files
(
    id           BIGSERIAL PRIMARY KEY,
    public_id    VARCHAR(16)                                   NOT NULL UNIQUE,
    folder_id    INT REFERENCES folders (id) ON DELETE CASCADE NOT NULL,

    -- encryption metadata about file, file stored in storage by `storage_path`
    data_meta    JSONB                                         NOT NULL,

    meta         JSONB                                         NOT NULL,
    storage_path UUID                                          NOT NULL UNIQUE,
    file_size    BIGINT                                        NOT NULL,
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP         NOT NULL
);

CREATE INDEX idx_folders_public_id ON folders (public_id);
CREATE INDEX idx_files_public_id ON files (public_id);
CREATE INDEX idx_files_folder_id ON files (folder_id);
CREATE INDEX IF NOT EXISTS idx_files_folder_created_at ON files (folder_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_folders_expired_at ON folders (expired_at) WHERE expired_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_files_meta_lookup ON files (public_id, data_meta, meta);
