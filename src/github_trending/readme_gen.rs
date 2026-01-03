use super::card::Card;
use crate::locales::get_resources;
use crate::models::Repository;
use anyhow::Result;
use chrono::Utc;
use log::info;
use std::path::Path;

pub struct ReadmeGenerator;

impl ReadmeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成当天的 README
    pub fn generate_daily_readme(
        &self,
        date: &str,
        categories: &[(String, Vec<(Repository, Card)>)], // (category_name, repos_with_cards)
        output_dir: &Path,
        locale: &str, // "en" or "zh"
    ) -> Result<String> {
        let is_cn = locale == "zh";
        let text = get_resources(locale);

        let mut content = String::new();

        // 标题和说明
        content.push_str(&format!("# {} - {}\n\n", text.title_prefix, date));
        content.push_str(&format!("{}\n\n", text.description));

        // 统计信息
        let total_repos: usize = categories.iter().map(|(_, repos)| repos.len()).sum();
        content.push_str(&format!("## {}\n\n", text.highlights_title));
        content.push_str(&format!("| {} | {} |\n", text.table_stat, text.table_value));
        content.push_str("|--------|------|\n");
        content.push_str(&format!("| {} | **{}** |\n", text.stat_items, total_repos));
        content.push_str(&format!(
            "| {} | {} |\n\n",
            text.stat_time,
            Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));

        // 每个分类
        for (category_name, repos) in categories {
            if repos.is_empty() {
                continue;
            }

            content.push_str("---\n\n");
            let display_name = self.format_category_name(category_name, locale);
            content.push_str(&format!(
                "## {} {}\n\n",
                self.get_category_emoji(category_name),
                display_name
            ));

            // 仓库表格
            for (idx, (repo, _card)) in repos.iter().enumerate() {
                // 项目标题
                content.push_str(&format!(
                    "### {}. [{}]({})\n\n",
                    idx + 1,
                    repo.name,
                    repo.html_url
                ));

                // 统计信息表格
                content.push_str(&format!(
                    "| {} | {} |\n",
                    text.table_indicator, text.table_val
                ));
                content.push_str("|------|----|\n");
                content.push_str(&format!("| ⭐ Stars | **{}** |\n", repo.stars));
                content.push_str(&format!("| 🍴 Forks | **{}** |\n", repo.forks));
                content.push_str(&format!(
                    "| 💻 Language | {} |\n",
                    repo.language.as_deref().unwrap_or("N/A")
                ));
                if !repo.topics.is_empty() {
                    let topics_str: Vec<String> = repo
                        .topics
                        .iter()
                        .take(5) // 最多显示5个标签
                        .map(|t| format!("`{}`", t))
                        .collect();
                    content.push_str(&format!("| 🏷️ Tags | {} |\n", topics_str.join(" ")));
                }

                // New: Stars Today (Strict Priority)
                if let Some(today) = repo.stars_today {
                    content.push_str(&format!("| {} | **{}** |\n", text.stars_today_label, today));
                }

                content.push_str("\n");

                // 项目描述
                if let Some(desc) = &repo.description {
                    content.push_str(&format!("**{}:** {}\n\n", text.desc_label, desc));
                }

                // 卡片图片
                let image_path = format!(
                    "{}_{}_{}.png",
                    date,
                    category_name,
                    repo.name.replace("/", "_")
                );
                content.push_str(&format!("![{}]({})\n\n", repo.name, image_path));
            }
        }

        // RSS 订阅链接
        content.push_str("---\n\n");
        content.push_str(&format!("## {}\n\n", text.rss_title));
        content.push_str(&format!("{}\n\n", text.rss_desc));

        // Link to daily-top.xml (Relative path from docs/rss/YYYY/MM-DD to docs/rss/daily-top.xml)
        // Path is ../../../daily-top.xml
        content.push_str(&format!(
            "- 🔔 [{}] (../../daily-top.xml)\n",
            text.rss_daily_xml_title
        ));

        // Link to Current Day's Report (Markdown)
        // Path is ../../../GITHUB_TODAY.md or ../../../GITHUB_TODAY_CN.md
        let daily_report_filename = if is_cn {
            "GITHUB_TODAY_CN.md"
        } else {
            "GITHUB_TODAY.md"
        };
        content.push_str(&format!(
            "- 🔔 [{}] (../../{})\n",
            text.rss_daily_report_title, daily_report_filename
        ));

        // Category feeds
        for (category_name, _) in categories {
            let display_name = self.format_category_name(category_name, locale);
            content.push_str(&format!(
                "- 🔔 [{}](../../{}.xml)\n",
                display_name, category_name
            ));
        }

        content.push_str("\n---\n\n");
        content.push_str(&format!(
            "{} {}\n",
            text.footer,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // File naming logic based on locale
        let filename = if is_cn { "README_CN.md" } else { "README.md" };
        let readme_path = output_dir.join(filename);
        std::fs::write(&readme_path, &content)?;
        info!("Generated {} at {:?}", filename, readme_path);

        // Update GITHUB_TODAY only for default (English) or both?
        let today_filename = if is_cn {
            "GITHUB_TODAY_CN.md"
        } else {
            "GITHUB_TODAY.md"
        };
        let today_path = output_dir.join(today_filename);
        std::fs::write(&today_path, &content)?;
        info!("Generated {}: {:?}", today_filename, today_path);

        Ok(content)
    }

    pub fn generate_landing_readme(&self, output_path: &Path, locale: &str) -> Result<()> {
        let text = get_resources(locale);
        let mut content = String::new();

        // Title & Subtitle
        content.push_str(&format!("# {}\n\n", text.landing_title));
        content.push_str(&format!("> {}\n\n", text.landing_subtitle));

        // Today's Picks Section
        content.push_str(&format!("## {}\n\n", text.landing_today_title));
        content.push_str(&format!("{}\n\n", text.landing_today_desc));

        // Link to Today's Report
        let daily_report_filename = if locale == "zh" {
            "GITHUB_TODAY_CN.md"
        } else {
            "GITHUB_TODAY.md"
        };
        // Link points to docs/rss/GITHUB_TODAY*.md from root
        content.push_str(&format!(
            "**[{}]({})**\n\n",
            text.landing_today_link,
            format!("docs/rss/{}", daily_report_filename)
        ));

        // RSS Subscription
        content.push_str(&format!("## {}\n\n", text.landing_rss_title));
        content.push_str(&format!("{}\n\n", text.landing_rss_desc));
        content.push_str(&format!(
            "- **{}**: [docs/rss/daily-top.xml](docs/rss/daily-top.xml)\n\n",
            text.landing_rss_xml_label
        ));

        // Features
        content.push_str(&format!("## {}\n\n", text.landing_features_title));
        content.push_str(&format!("- {}\n", text.landing_feature_algo));
        content.push_str(&format!("- {}\n", text.landing_feature_daily));
        content.push_str(&format!("- {}\n", text.landing_feature_card));
        content.push_str(&format!("- {}\n\n", text.landing_feature_rss));

        // History
        content.push_str(&format!("## {}\n\n", text.landing_history_title));
        content.push_str(&format!("{}\n", text.landing_history_desc));

        // Write to file
        std::fs::write(output_path, content)?;
        info!("Generated Landing Page: {:?}", output_path);

        Ok(())
    }

    fn get_category_emoji(&self, name: &str) -> &str {
        match name {
            "backend" => "🔧",
            "frontend" => "🎨",
            "mobile" => "📱",
            "ai-ml" => "🤖",
            "daily-top" => "🌟",
            _ => "📦",
        }
    }

    fn format_category_name(&self, name: &str, locale: &str) -> String {
        let text = get_resources(locale);
        match name {
            "backend" => text.cat_backend.to_string(),
            "frontend" => text.cat_frontend.to_string(),
            "mobile" => text.cat_mobile.to_string(),
            "ai-ml" => text.cat_ai_ml.to_string(),
            "daily-top" => text.cat_daily_top.to_string(),
            _ => name.to_string(),
        }
    }
}
