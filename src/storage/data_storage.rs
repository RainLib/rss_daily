use crate::models::Repository;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyData {
    pub date: String,
    pub name: String,
    pub repositories: Vec<Repository>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct DataStorage {
    base_dir: PathBuf,
}

impl DataStorage {
    pub fn new(base_path: &str) -> Result<Self> {
        let base_dir = PathBuf::from(base_path);
        std::fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    /// 保存每日数据
    pub fn save_daily_data(&self, date: &str, name: &str, repos: &[Repository]) -> Result<PathBuf> {
        let data = DailyData {
            date: date.to_string(),
            name: name.to_string(),
            repositories: repos.to_vec(),
            created_at: Utc::now(),
        };

        // 解析日期，构建目录结构: base_dir/YYYY/MM-DD/name.json
        // date 格式预期为 YYYY-MM-DD
        let parts: Vec<&str> = date.split('-').collect();
        let (year, month_day) = if parts.len() == 3 {
            (parts[0], format!("{}-{}", parts[1], parts[2]))
        } else {
            // Fallback
            ("unknown", date.to_string())
        };

        // 创建年份和日期目录
        let archive_dir = self.base_dir.join(year).join(&month_day);
        std::fs::create_dir_all(&archive_dir)?;

        // 文件名格式：name.json (或者保持 date_name.json，用户仅要求目录结构)
        // 用户要求: @[data/github_trending] 下的数据也要按照日期目录啦进行归档/2026/01-02/ 目录结构
        let filename = format!("{}_{}.json", date, name);
        let file_path = archive_dir.join(&filename);

        let content = serde_json::to_string_pretty(&data)?;
        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    /// 加载指定日期的数据
    pub fn load_daily_data(&self, date: &str, name: &str) -> Result<DailyData> {
        // Compute path same as save
        let parts: Vec<&str> = date.split('-').collect();
        let (year, month_day) = if parts.len() == 3 {
            (parts[0], format!("{}-{}", parts[1], parts[2]))
        } else {
            ("unknown", date.to_string())
        };

        let filename = format!("{}_{}.json", date, name);
        let file_path = self.base_dir.join(year).join(month_day).join(&filename);

        let content = std::fs::read_to_string(&file_path)?;
        let data: DailyData = serde_json::from_str(&content)?;

        Ok(data)
    }

    /// 列出所有日期数据
    pub fn list_dates(&self) -> Result<Vec<String>> {
        let mut dates = Vec::new();
        // Naive recursive search or just rely on structure.
        // Since we know structure is base/YYYY/MM-DD
        if !self.base_dir.exists() {
            return Ok(dates);
        }

        for year_entry in std::fs::read_dir(&self.base_dir)? {
            let year_entry = year_entry?;
            let year_path = year_entry.path();
            if year_path.is_dir() {
                for day_entry in std::fs::read_dir(year_path)? {
                    let day_entry = day_entry?;
                    let day_path = day_entry.path();
                    if day_path.is_dir() {
                        // Look for json files in here
                        for file_entry in std::fs::read_dir(day_path)? {
                            let file_entry = file_entry?;
                            let path = file_entry.path();
                            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                if filename.ends_with(".json") {
                                    if let Some(date_part) = filename.split('_').next() {
                                        if !dates.contains(&date_part.to_string()) {
                                            dates.push(date_part.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        dates.sort();
        Ok(dates)
    }

    /// 加载所有历史数据
    pub fn load_all_history(&self) -> Result<Vec<Repository>> {
        let mut all_repos = Vec::new();

        if !self.base_dir.exists() {
            return Ok(all_repos);
        }

        // Recursive walk
        let mut dirs_to_visit = vec![self.base_dir.clone()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            for entry in std::fs::read_dir(current_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else {
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(data) = serde_json::from_str::<DailyData>(&content) {
                                all_repos.extend(data.repositories);
                            }
                        }
                    }
                }
            }
        }

        Ok(all_repos)
    }
}
