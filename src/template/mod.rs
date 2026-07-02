use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ui::layout_tree::LayoutNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layout: LayoutNode,
    pub commands: Vec<TemplateCommand>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCommand {
    pub name: String,
    pub command: String,
    pub terminal_id: String,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub templates: HashMap<String, WorkspaceTemplate>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        let mut config = TemplateConfig {
            templates: HashMap::new(),
        };
        config.add_default_templates();
        config
    }
}

impl TemplateConfig {
    pub fn new() -> Self {
        TemplateConfig::default()
    }

    fn add_default_templates(&mut self) {
        // 前端开发模板
        self.add_template(WorkspaceTemplate {
            id: "frontend-dev".to_string(),
            name: "前端开发".to_string(),
            description: "适合前端开发的工作区模板".to_string(),
            layout: LayoutNode::new_split(
                crate::ui::layout_tree::SplitDirection::Vertical,
                vec![
                    LayoutNode::new_pane("terminal-1", "代码编辑器"),
                    LayoutNode::new_split(
                        crate::ui::layout_tree::SplitDirection::Horizontal,
                        vec![
                            LayoutNode::new_pane("terminal-2", "终端"),
                            LayoutNode::new_pane("terminal-3", "浏览器"),
                        ],
                    ),
                ],
            ),
            commands: vec![
                TemplateCommand {
                    name: "启动开发服务器".to_string(),
                    command: "npm run dev".to_string(),
                    terminal_id: "terminal-2".to_string(),
                    working_directory: None,
                },
            ],
            tags: vec!["frontend".to_string(), "web".to_string()],
        });

        // 后端开发模板
        self.add_template(WorkspaceTemplate {
            id: "backend-dev".to_string(),
            name: "后端开发".to_string(),
            description: "适合后端开发的工作区模板".to_string(),
            layout: LayoutNode::new_split(
                crate::ui::layout_tree::SplitDirection::Horizontal,
                vec![
                    LayoutNode::new_pane("terminal-1", "代码编辑器"),
                    LayoutNode::new_split(
                        crate::ui::layout_tree::SplitDirection::Horizontal,
                        vec![
                            LayoutNode::new_pane("terminal-2", "服务器"),
                            LayoutNode::new_pane("terminal-3", "数据库"),
                        ],
                    ),
                ],
            ),
            commands: vec![
                TemplateCommand {
                    name: "启动服务器".to_string(),
                    command: "cargo run".to_string(),
                    terminal_id: "terminal-2".to_string(),
                    working_directory: None,
                },
            ],
            tags: vec!["backend".to_string(), "rust".to_string()],
        });

        // 数据科学模板
        self.add_template(WorkspaceTemplate {
            id: "data-science".to_string(),
            name: "数据科学".to_string(),
            description: "适合数据科学的工作区模板".to_string(),
            layout: LayoutNode::new_split(
                crate::ui::layout_tree::SplitDirection::Horizontal,
                vec![
                    LayoutNode::new_pane("terminal-1", "Jupyter Notebook"),
                    LayoutNode::new_pane("terminal-2", "终端"),
                ],
            ),
            commands: vec![
                TemplateCommand {
                    name: "启动 Jupyter".to_string(),
                    command: "jupyter notebook".to_string(),
                    terminal_id: "terminal-1".to_string(),
                    working_directory: None,
                },
            ],
            tags: vec!["data".to_string(), "python".to_string()],
        });
    }

    pub fn add_template(&mut self, template: WorkspaceTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn remove_template(&mut self, template_id: &str) -> Option<WorkspaceTemplate> {
        self.templates.remove(template_id)
    }

    pub fn get_template(&self, template_id: &str) -> Option<&WorkspaceTemplate> {
        self.templates.get(template_id)
    }

    pub fn list_templates(&self) -> Vec<&WorkspaceTemplate> {
        self.templates.values().collect()
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&WorkspaceTemplate> {
        self.templates
            .values()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn save(&self) -> Result<()> {
        let config_path = crate::config::paths::get_config_dir().join("templates.json");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let config_path = crate::config::paths::get_config_dir().join("templates.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: TemplateConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(TemplateConfig::default())
        }
    }
}
