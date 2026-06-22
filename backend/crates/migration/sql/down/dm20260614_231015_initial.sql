DROP INDEX IF EXISTS idx_files_hash;
DROP INDEX IF EXISTS idx_files_folder_id;
DROP INDEX IF EXISTS idx_files_long_id;
DROP INDEX IF EXISTS idx_folders_long_id;

DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS folders;