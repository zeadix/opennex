use super::layout_tree::{LayoutNode, SplitDirection};

#[derive(Debug, Clone)]
pub struct LayoutPreset {
    pub name: String,
    pub description: String,
    pub layout: LayoutNode,
}

pub fn get_presets() -> Vec<LayoutPreset> {
    vec![
        LayoutPreset {
            name: "single".to_string(),
            description: "单窗口".to_string(),
            layout: LayoutNode::new_pane("terminal-1", "终端 1"),
        },
        LayoutPreset {
            name: "horizontal-split".to_string(),
            description: "水平分屏".to_string(),
            layout: LayoutNode::new_split(
                SplitDirection::Horizontal,
                vec![
                    LayoutNode::new_pane("terminal-1", "终端 1"),
                    LayoutNode::new_pane("terminal-2", "终端 2"),
                ],
            ),
        },
        LayoutPreset {
            name: "vertical-split".to_string(),
            description: "垂直分屏".to_string(),
            layout: LayoutNode::new_split(
                SplitDirection::Vertical,
                vec![
                    LayoutNode::new_pane("terminal-1", "终端 1"),
                    LayoutNode::new_pane("terminal-2", "终端 2"),
                ],
            ),
        },
        LayoutPreset {
            name: "grid-4".to_string(),
            description: "四宫格".to_string(),
            layout: LayoutNode::new_split(
                SplitDirection::Horizontal,
                vec![
                    LayoutNode::new_split(
                        SplitDirection::Vertical,
                        vec![
                            LayoutNode::new_pane("terminal-1", "终端 1"),
                            LayoutNode::new_pane("terminal-2", "终端 2"),
                        ],
                    ),
                    LayoutNode::new_split(
                        SplitDirection::Vertical,
                        vec![
                            LayoutNode::new_pane("terminal-3", "终端 3"),
                            LayoutNode::new_pane("terminal-4", "终端 4"),
                        ],
                    ),
                ],
            ),
        },
        LayoutPreset {
            name: "main-side".to_string(),
            description: "主侧布局".to_string(),
            layout: LayoutNode::new_split(
                SplitDirection::Vertical,
                vec![
                    LayoutNode::new_pane("terminal-1", "终端 1"),
                    LayoutNode::new_split(
                        SplitDirection::Horizontal,
                        vec![
                            LayoutNode::new_pane("terminal-2", "终端 2"),
                            LayoutNode::new_pane("terminal-3", "终端 3"),
                        ],
                    ),
                ],
            ),
        },
    ]
}

pub fn get_preset_by_name(name: &str) -> Option<LayoutPreset> {
    get_presets().into_iter().find(|p| p.name == name)
}
