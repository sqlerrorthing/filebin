DROP INDEX IF EXISTS idx_files_folder_id;
DROP INDEX IF EXISTS idx_folders_public_id;
DROP INDEX IF EXISTS idx_files_public_id;
DROP INDEX IF EXISTS idx_encrypted_blobs_id;
DROP INDEX IF EXISTS idx_files_folder_created_at;
DROP INDEX IF EXISTS idx_folders_expired_at;
DROP INDEX IF EXISTS idx_files_meta_lookup;

DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS folders;