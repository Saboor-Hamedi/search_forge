use crate::db::schema::CREATE_FILES_TABLE;
use crate::indexer::app_scanner::SearchResultType;
use crate::indexer::scanner::FileRecord;
use rusqlite::{params, Connection, Result};

pub struct Database {
    conn: Connection,
}

#[allow(dead_code)]
impl Database {
    pub fn init_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_FILES_TABLE)?;
        Ok(Self { conn })
    }

    #[allow(dead_code)]
    pub fn insert_file(&mut self, record: &FileRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO files (name, path, size, is_dir) VALUES (?1, ?2, ?3, ?4)",
            params![
                record.name,
                record.path,
                record.size as i64,
                record.is_dir as i32
            ],
        )?;
        Ok(())
    }

    pub fn insert_batch(&mut self, records: &[FileRecord]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO files (name, path, size, is_dir) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for record in records {
                stmt.execute(params![
                    record.name,
                    record.path,
                    record.size as i64,
                    record.is_dir as i32
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<FileRecord>> {
        let pattern = format!("%{}%", query);
        let has_slash = query.contains('/') || query.contains('\\');
        let sql = if has_slash {
            "SELECT name, path, size, is_dir FROM files
             WHERE name LIKE ?1 OR path LIKE ?1
             ORDER BY (CASE WHEN name LIKE ?1 THEN 0 ELSE 1 END), is_dir ASC, name ASC
             LIMIT ?2"
        } else {
            "SELECT name, path, size, is_dir FROM files
             WHERE name LIKE ?1
             ORDER BY is_dir ASC, name ASC
             LIMIT ?2"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            let is_dir: bool = row.get::<_, i32>(3)? != 0;
            Ok(FileRecord {
                name: row.get(0)?,
                path: row.get(1)?,
                size: row.get::<_, i64>(2)? as u64,
                is_dir,
                item_type: if is_dir {
                    SearchResultType::Folder
                } else {
                    SearchResultType::File
                },
                app_metadata: None,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_recent_files(&self, limit: usize) -> Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, path, size, is_dir FROM files
             WHERE is_dir = 0
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let is_dir: bool = row.get::<_, i32>(3)? != 0;
            Ok(FileRecord {
                name: row.get(0)?,
                path: row.get(1)?,
                size: row.get::<_, i64>(2)? as u64,
                is_dir,
                item_type: if is_dir {
                    SearchResultType::Folder
                } else {
                    SearchResultType::File
                },
                app_metadata: None,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_search_and_recent() {
        let mut db = Database::init_in_memory().unwrap();
        let records = vec![
            FileRecord {
                name: "invoice_99.pdf".to_string(),
                path: "C:/docs/invoice_99.pdf".to_string(),
                size: 1024,
                is_dir: false,
                item_type: SearchResultType::File,
                app_metadata: None,
            },
            FileRecord {
                name: "notes.md".to_string(),
                path: "C:/docs/notes.md".to_string(),
                size: 2048,
                is_dir: false,
                item_type: SearchResultType::File,
                app_metadata: None,
            },
            FileRecord {
                name: "code".to_string(),
                path: "C:/projects/code".to_string(),
                size: 0,
                is_dir: true,
                item_type: SearchResultType::Folder,
                app_metadata: None,
            },
        ];

        let count = db.insert_batch(&records).unwrap();
        assert_eq!(count, 3);

        // Search test
        let results = db.search("99", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "invoice_99.pdf");

        // Recent files test (should return only files, no dirs)
        let recent = db.get_recent_files(5).unwrap();
        assert_eq!(recent.len(), 2);
    }
}
