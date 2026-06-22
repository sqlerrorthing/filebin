CREATE TABLE folders
(
    id             SERIAL PRIMARY KEY,
    public_id      VARCHAR(8)                            NOT NULL UNIQUE,
    encrypted_name TEXT                                  NOT NULL,
    expired_at     TIMESTAMPTZ,
    created_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE files
(
    id                  SERIAL PRIMARY KEY,
    public_id           VARCHAR(16)                                   NOT NULL UNIQUE,
    folder_id           INT REFERENCES folders (id) ON DELETE CASCADE NOT NULL,
    storage_path        UUID                                          NOT NULL UNIQUE,
    encrypted_path      TEXT                                          NOT NULL,
    encrypted_mime_type TEXT                                          NOT NULL,
    encrypted_file_hash VARCHAR(64)                                   NOT NULL,
    file_size           BIGINT                                        NOT NULL,
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP         NOT NULL
);

CREATE INDEX idx_folders_public_id ON folders (public_id);
CREATE INDEX idx_files_public_id ON files (public_id);
CREATE INDEX idx_files_folder_id ON files (folder_id);
CREATE INDEX idx_files_hash ON files (encrypted_file_hash);
