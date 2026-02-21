use crate::models::*;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self> {
        std::fs::create_dir_all(app_data_dir).ok();
        let db_path = app_data_dir.join("novelparser.db");
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS novels (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                source_type TEXT NOT NULL,
                enabled_dimensions TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chapters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                novel_id TEXT NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
                chapter_index INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                analysis TEXT
            );

            CREATE TABLE IF NOT EXISTS novel_summaries (
                novel_id TEXT PRIMARY KEY REFERENCES novels(id) ON DELETE CASCADE,
                summary TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS summary_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                novel_id TEXT NOT NULL REFERENCES novels(id) ON DELETE CASCADE,
                layer INTEGER NOT NULL,
                group_index INTEGER NOT NULL,
                content TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chapters_novel ON chapters(novel_id, chapter_index);
            ",
        )?;
        Ok(())
    }

    // ---- Novel CRUD ----

    pub fn save_novel(&self, novel: &Novel) -> Result<()> {
        let source_type_json = serde_json::to_string(&novel.source_type).unwrap_or_default();
        let dims_json = serde_json::to_string(&novel.enabled_dimensions).unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO novels (id, title, source_type, enabled_dimensions, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                novel.id,
                novel.title,
                source_type_json,
                dims_json,
                novel.created_at
            ],
        )?;
        Ok(())
    }

    pub fn load_novel(&self, id: &str) -> Result<Novel> {
        self.conn.query_row(
            "SELECT id, title, source_type, enabled_dimensions, created_at FROM novels WHERE id = ?1",
            params![id],
            |row| {
                let source_type_str: String = row.get(2)?;
                let dims_str: String = row.get(3)?;
                Ok(Novel {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    source_type: serde_json::from_str(&source_type_str).unwrap_or(SourceType::Epub(String::new())),
                    enabled_dimensions: serde_json::from_str(&dims_str).unwrap_or_default(),
                    created_at: row.get(4)?,
                })
            },
        )
    }

    pub fn list_novels(&self) -> Result<Vec<NovelMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.title, n.created_at,
                    COUNT(c.id) as chapter_count,
                    COUNT(c.analysis) as analyzed_count
             FROM novels n
             LEFT JOIN chapters c ON c.novel_id = n.id
             GROUP BY n.id
             ORDER BY n.created_at DESC",
        )?;
        let results = stmt
            .query_map([], |row| {
                Ok(NovelMeta {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    chapter_count: row.get::<_, i64>(3)? as usize,
                    analyzed_count: row.get::<_, i64>(4)? as usize,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(results)
    }

    pub fn delete_novel(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM novels WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ---- Chapter CRUD ----

    pub fn save_chapter(&self, chapter: &Chapter) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO chapters (novel_id, chapter_index, title, content, analysis)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                chapter.novel_id,
                chapter.index as i64,
                chapter.title,
                chapter.content,
                chapter
                    .analysis
                    .as_ref()
                    .map(|a| serde_json::to_string(a).unwrap_or_default()),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_chapter_metas(&self, novel_id: &str) -> Result<Vec<ChapterMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chapter_index, title, analysis, LENGTH(content) as content_len
             FROM chapters WHERE novel_id = ?1 ORDER BY chapter_index",
        )?;
        let results = stmt
            .query_map(params![novel_id], |row| {
                let analysis_str: Option<String> = row.get(3)?;
                let content_len: i64 = row.get(4)?;
                Ok(ChapterMeta {
                    id: row.get(0)?,
                    index: row.get::<_, i64>(1)? as usize,
                    title: row.get(2)?,
                    has_analysis: analysis_str.is_some(),
                    token_estimate: (content_len as f64 * 1.5) as usize,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(results)
    }

    pub fn load_chapter(&self, chapter_id: i64) -> Result<Chapter> {
        self.conn.query_row(
            "SELECT id, novel_id, chapter_index, title, content, analysis
             FROM chapters WHERE id = ?1",
            params![chapter_id],
            |row| {
                let analysis_str: Option<String> = row.get(5)?;
                Ok(Chapter {
                    id: Some(row.get(0)?),
                    novel_id: row.get(1)?,
                    index: row.get::<_, i64>(2)? as usize,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    analysis: analysis_str.and_then(|s| serde_json::from_str(&s).ok()),
                })
            },
        )
    }

    pub fn load_chapter_content(&self, chapter_id: i64) -> Result<String> {
        self.conn.query_row(
            "SELECT content FROM chapters WHERE id = ?1",
            params![chapter_id],
            |row| row.get(0),
        )
    }

    pub fn save_chapter_analysis(&self, chapter_id: i64, analysis: &ChapterAnalysis) -> Result<()> {
        let json = serde_json::to_string(analysis).unwrap_or_default();
        self.conn.execute(
            "UPDATE chapters SET analysis = ?1 WHERE id = ?2",
            params![json, chapter_id],
        )?;
        Ok(())
    }

    pub fn delete_chapter(&self, chapter_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM chapters WHERE id = ?1", params![chapter_id])?;
        Ok(())
    }

    pub fn delete_chapters(&self, chapter_ids: &[i64]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for &id in chapter_ids {
            tx.execute("DELETE FROM chapters WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn clear_chapter_analysis(&self, chapter_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE chapters SET analysis = NULL WHERE id = ?1",
            params![chapter_id],
        )?;
        Ok(())
    }

    pub fn load_previous_chapter_analysis(
        &self,
        novel_id: &str,
        current_index: usize,
    ) -> Result<Option<ChapterAnalysis>> {
        let result = self.conn.query_row(
            "SELECT analysis FROM chapters 
             WHERE novel_id = ?1 AND chapter_index < ?2 AND analysis IS NOT NULL 
             ORDER BY chapter_index DESC LIMIT 1",
            params![novel_id, current_index as i64],
            |row| {
                let analysis_str: String = row.get(0)?;
                Ok(serde_json::from_str(&analysis_str).ok())
            },
        );

        match result {
            Ok(Some(analysis)) => Ok(Some(analysis)),
            Ok(None) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn load_all_previous_analyses(
        &self,
        novel_id: &str,
        current_index: usize,
    ) -> Result<Vec<(usize, ChapterAnalysis)>> {
        let mut stmt = self.conn.prepare(
            "SELECT chapter_index, analysis FROM chapters 
             WHERE novel_id = ?1 AND chapter_index < ?2 AND analysis IS NOT NULL 
             ORDER BY chapter_index ASC",
        )?;

        let results = stmt.query_map(params![novel_id, current_index as i64], |row| {
            let index: i64 = row.get(0)?;
            let analysis_str: String = row.get(1)?;
            let analysis: Option<ChapterAnalysis> = serde_json::from_str(&analysis_str).ok();
            Ok((index as usize, analysis))
        })?;

        let mut analyses = Vec::new();
        for res in results {
            if let Ok((index, Some(analysis))) = res {
                analyses.push((index, analysis));
            }
        }

        Ok(analyses)
    }

    // ---- Novel Summary ----

    pub fn save_novel_summary(&self, novel_id: &str, summary: &NovelSummary) -> Result<()> {
        let json = serde_json::to_string(summary).unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO novel_summaries (novel_id, summary) VALUES (?1, ?2)",
            params![novel_id, json],
        )?;
        Ok(())
    }

    pub fn load_novel_summary(&self, novel_id: &str) -> Result<Option<NovelSummary>> {
        let result = self.conn.query_row(
            "SELECT summary FROM novel_summaries WHERE novel_id = ?1",
            params![novel_id],
            |row| {
                let json: String = row.get(0)?;
                Ok(serde_json::from_str(&json).ok())
            },
        );
        match result {
            Ok(Some(summary)) => {
                let json_summary: NovelSummary = summary;
                Ok(Some(json_summary))
            }
            Ok(None) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn clear_novel_summary(&self, novel_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM novel_summaries WHERE novel_id = ?1",
            params![novel_id],
        )?;
        Ok(())
    }

    // ---- Settings ----

    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn load_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn save_llm_config(&self, config: &LlmConfig) -> Result<()> {
        let json = serde_json::to_string(config).unwrap_or_default();
        self.save_setting("llm_config", &json)
    }

    pub fn load_llm_config(&self) -> Result<LlmConfig> {
        match self.load_setting("llm_config")? {
            Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
            None => Ok(LlmConfig::default()),
        }
    }

    // ---- Summary Cache ----

    pub fn save_summary_cache(
        &self,
        novel_id: &str,
        layer: i32,
        group_index: i32,
        content: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO summary_cache (novel_id, layer, group_index, content) VALUES (?1, ?2, ?3, ?4)",
            params![novel_id, layer, group_index, content],
        )?;
        Ok(())
    }

    pub fn clear_summary_cache(&self, novel_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM summary_cache WHERE novel_id = ?1",
            params![novel_id],
        )?;
        Ok(())
    }
}
