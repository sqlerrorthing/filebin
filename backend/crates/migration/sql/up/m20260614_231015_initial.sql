CREATE TABLE folders
(
    id         SERIAL PRIMARY KEY,
    long_id    VARCHAR(32)                           NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE files
(
    id                  SERIAL PRIMARY KEY,
    long_id             VARCHAR(32)                           NOT NULL UNIQUE,
    folder_id           INT REFERENCES folders (id) ON DELETE CASCADE,

    encrypted_name      TEXT                                  NOT NULL,
    encrypted_mime_type TEXT                                  NOT NULL,
    encrypted_file_hash VARCHAR(64)                           NOT NULL,
    encrypted_file_size BIGINT                                NOT NULL,
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_folders_long_id ON folders (long_id);
CREATE INDEX idx_files_long_id ON files (long_id);
CREATE INDEX idx_files_folder_id ON files (folder_id);
CREATE INDEX idx_files_hash ON files (encrypted_file_hash);
