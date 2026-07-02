use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    Split {
        direction: SplitDirection,
        ratio: f32,
        children: Vec<LayoutNode>,
    },
    Pane {
        id: String,
        title: String,
    },
}

impl LayoutNode {
    pub fn new_pane(id: &str, title: &str) -> Self {
        LayoutNode::Pane {
            id: id.to_string(),
            title: title.to_string(),
        }
    }

    pub fn new_split(direction: SplitDirection, children: Vec<LayoutNode>) -> Self {
        LayoutNode::Split {
            direction,
            ratio: 0.5,
            children,
        }
    }

    pub fn split_pane(&mut self, pane_id: &str, direction: SplitDirection, new_pane_id: &str, new_title: &str) -> bool {
        match self {
            LayoutNode::Pane { id, .. } => {
                if id == pane_id {
                    let old_pane = self.clone();
                    let new_pane = LayoutNode::new_pane(new_pane_id, new_title);
                    *self = LayoutNode::Split {
                        direction,
                        ratio: 0.5,
                        children: vec![old_pane, new_pane],
                    };
                    return true;
                }
                false
            }
            LayoutNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    if child.split_pane(pane_id, direction.clone(), new_pane_id, new_title) {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn remove_pane(&mut self, pane_id: &str) -> bool {
        match self {
            LayoutNode::Pane { id, .. } => id == pane_id,
            LayoutNode::Split { children, .. } => {
                if let Some(pos) = children.iter_mut().position(|c| c.remove_pane(pane_id)) {
                    children.remove(pos);
                    if children.len() == 1 {
                        *self = children.remove(0);
                    }
                    return true;
                }
                false
            }
        }
    }

    pub fn find_pane(&self, pane_id: &str) -> Option<&LayoutNode> {
        match self {
            LayoutNode::Pane { id, .. } => {
                if id == pane_id {
                    Some(self)
                } else {
                    None
                }
            }
            LayoutNode::Split { children, .. } => {
                for child in children {
                    if let Some(found) = child.find_pane(pane_id) {
                        return Some(found);
                    }
                }
                None
            }
        }
    }

    pub fn get_all_pane_ids(&self) -> Vec<String> {
        match self {
            LayoutNode::Pane { id, .. } => vec![id.clone()],
            LayoutNode::Split { children, .. } => {
                let mut ids = Vec::new();
                for child in children {
                    ids.extend(child.get_all_pane_ids());
                }
                ids
            }
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::new_pane("terminal-1", "终端 1")
    }
}
