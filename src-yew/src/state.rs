use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;
use url::Url;

#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
pub struct Response {
    pub url: String,
    pub status: u16,
    pub ok: bool,
    pub headers: HashMap<String, String>,
    pub raw_headers: HashMap<String, Vec<String>>,
    pub data: String,
    #[serde(default)]
    pub formatted: bool,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            url: String::new(),
            status: 200,
            ok: true,
            headers: HashMap::new(),
            raw_headers: HashMap::new(),
            data: String::new(),
            formatted: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MethodEnum {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
}

impl MethodEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            MethodEnum::Get => "GET",
            MethodEnum::Post => "POST",
            MethodEnum::Put => "PUT",
            MethodEnum::Patch => "PATCH",
            MethodEnum::Delete => "DELETE",
            MethodEnum::Options => "OPTIONS",
            MethodEnum::Head => "HEAD",
            MethodEnum::Trace => "TRACE",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            MethodEnum::Get => "get",
            MethodEnum::Post => "post",
            MethodEnum::Put => "put",
            MethodEnum::Patch => "patch",
            MethodEnum::Delete => "delete",
            MethodEnum::Options => "options",
            MethodEnum::Head => "head",
            MethodEnum::Trace => "trace",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_ascii_uppercase().as_str() {
            "GET" => Some(MethodEnum::Get),
            "POST" => Some(MethodEnum::Post),
            "PUT" => Some(MethodEnum::Put),
            "PATCH" => Some(MethodEnum::Patch),
            "DELETE" => Some(MethodEnum::Delete),
            "OPTIONS" => Some(MethodEnum::Options),
            "HEAD" => Some(MethodEnum::Head),
            "TRACE" => Some(MethodEnum::Trace),
            _ => None,
        }
    }

    pub fn all() -> &'static [MethodEnum] {
        static METHODS: [MethodEnum; 8] = [
            MethodEnum::Get,
            MethodEnum::Post,
            MethodEnum::Put,
            MethodEnum::Patch,
            MethodEnum::Delete,
            MethodEnum::Options,
            MethodEnum::Head,
            MethodEnum::Trace,
        ];
        &METHODS
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Header {
    pub enable: bool,
    pub key: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Param {
    pub enable: bool,
    pub key: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TabContent {
    pub method: MethodEnum,
    pub url: String,
    pub body: String,
    pub body_formatted: bool,
    pub headers: Vec<Header>,
    pub params: Vec<Param>,
    pub response: Response,
}

impl TabContent {
    pub fn from_node(content: &TabContent) -> Self {
        Self {
            method: content.method,
            url: content.url.clone(),
            body: content.body.clone(),
            body_formatted: false,
            headers: content.headers.clone(),
            params: content.params.clone(),
            response: Response::default(),
        }
    }
}

impl Default for TabContent {
    fn default() -> Self {
        Self {
            method: MethodEnum::Get,
            url: "/".to_string(),
            body: String::new(),
            body_formatted: false,
            headers: vec![
                Header {
                    enable: true,
                    key: "Accept".to_string(),
                    value: "*/*".to_string(),
                },
                Header {
                    enable: true,
                    key: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
            params: vec![Param {
                enable: true,
                key: String::new(),
                value: String::new(),
            }],
            response: Response::default(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tab {
    pub label: String,
    pub content: TabContent,
    pub dirty: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TabState {
    pub active_tab_id: usize,
    pub tabs: Vec<Tab>,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            active_tab_id: 0,
            tabs: Vec::new(),
        }
    }
}

pub enum TabAction {
    AddTab,
    OpenTab { label: String, content: TabContent },
    CloseTab(usize),
    SetActive(usize),
    RenameTab { index: usize, label: String },
    SetDirty { index: usize, dirty: bool },
    UpdateMethod { index: usize, method: MethodEnum },
    UpdateUrl { index: usize, url: String },
    UpdateBody { index: usize, body: String },
    SetBodyState {
        index: usize,
        body: String,
        formatted: bool,
    },
    SetHeaders { index: usize, headers: Vec<Header> },
    UpdateUrlAndParams {
        index: usize,
        url: String,
        params: Vec<Param>,
    },
    SetResponse { index: usize, response: Response },
}

impl Reducible for TabState {
    type Action = TabAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            TabAction::AddTab => {
                state.tabs.push(Tab {
                    label: "/".to_string(),
                    content: TabContent::default(),
                    dirty: false,
                });
                state.active_tab_id = state.tabs.len().saturating_sub(1);
            }
            TabAction::OpenTab { label, content } => {
                state.tabs.push(Tab {
                    label,
                    content,
                    dirty: false,
                });
                state.active_tab_id = state.tabs.len().saturating_sub(1);
            }
            TabAction::CloseTab(index) => {
                if index < state.tabs.len() {
                    state.tabs.remove(index);
                }
                if state.tabs.is_empty() {
                    state.active_tab_id = 0;
                } else if state.active_tab_id >= state.tabs.len() {
                    state.active_tab_id = state.tabs.len().saturating_sub(1);
                } else if state.active_tab_id == index && index > 0 {
                    state.active_tab_id = index - 1;
                }
            }
            TabAction::SetActive(index) => {
                if index < state.tabs.len() {
                    state.active_tab_id = index;
                }
            }
            TabAction::RenameTab { index, label } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.label = label;
                }
            }
            TabAction::SetDirty { index, dirty } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.dirty = dirty;
                }
            }
            TabAction::UpdateMethod { index, method } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.method = method;
                    tab.dirty = true;
                }
            }
            TabAction::UpdateUrl { index, url } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.url = url;
                    tab.dirty = true;
                    if let Some(label) = tab_label_from_url(&tab.content.url) {
                        tab.label = label;
                    }
                }
            }
            TabAction::UpdateBody { index, body } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.body = body;
                    tab.content.body_formatted = false;
                    tab.dirty = true;
                }
            }
            TabAction::SetBodyState {
                index,
                body,
                formatted,
            } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.body = body;
                    tab.content.body_formatted = formatted;
                    tab.dirty = true;
                }
            }
            TabAction::SetHeaders { index, headers } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.headers = headers;
                    tab.dirty = true;
                }
            }
            TabAction::UpdateUrlAndParams { index, url, params } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.url = url;
                    tab.content.params = params;
                    tab.dirty = true;
                    if let Some(label) = tab_label_from_url(&tab.content.url) {
                        tab.label = label;
                    }
                }
            }
            TabAction::SetResponse { index, response } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.response = response;
                }
            }
        }
        Rc::new(state)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TreeNode {
    pub label: String,
    pub content: Option<TabContent>,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TreeState {
    pub root: TreeNode,
    pub servers: Vec<String>,
    pub selected_path: Option<Vec<usize>>,
    pub pending_delete: Option<PendingDelete>,
    pub pending_move: Option<PendingMove>,
    pub selected_server: Option<usize>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            root: TreeNode {
                label: "Root".to_string(),
                content: None,
                expanded: true,
                children: Vec::new(),
            },
            servers: Vec::new(),
            selected_path: None,
            pending_delete: None,
            pending_move: None,
            selected_server: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PendingDelete {
    pub path: Vec<usize>,
    pub label: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PendingMove {
    pub path: Vec<usize>,
    pub label: String,
}

pub enum TreeAction {
    SetExpanded { path: Vec<usize>, open: bool },
    AddServer { label: String },
    RemoveServer { index: usize },
    SetSelectedServer { index: usize },
    SetServers { servers: Vec<String> },
    SetTree { root: TreeNode, servers: Vec<String> },
    AddChild { path: Vec<usize>, node: TreeNode },
    ReplaceNode { path: Vec<usize>, node: TreeNode },
    Rename { path: Vec<usize>, label: String },
    SetSelected { path: Vec<usize> },
    RequestDelete { path: Vec<usize>, label: String },
    ClearPendingDelete,
    RemoveNode { path: Vec<usize> },
    RequestMove { path: Vec<usize>, label: String },
    ClearPendingMove,
    MoveNode { from: Vec<usize>, to: Vec<usize> },
}

impl Reducible for TreeState {
    type Action = TreeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            TreeAction::SetExpanded { path, open } => {
                set_expanded(&mut state.root, &path, open);
            }
            TreeAction::AddServer { label } => {
                state.servers.push(label);
                if state.selected_server.is_none() {
                    state.selected_server = Some(state.servers.len().saturating_sub(1));
                }
            }
            TreeAction::RemoveServer { index } => {
                if index < state.servers.len() {
                    state.servers.remove(index);
                }
                state.pending_move = None;
                state.pending_delete = None;
                state.selected_server =
                    adjust_selected_server(state.selected_server, index, state.servers.len());
            }
            TreeAction::SetSelectedServer { index } => {
                if index < state.servers.len() {
                    state.selected_server = Some(index);
                }
            }
            TreeAction::SetServers { servers } => {
                state.servers = servers;
                state.selected_server = if state.servers.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            TreeAction::SetTree { root, servers } => {
                state.root = root;
                state.servers = servers;
                state.selected_server = if state.servers.is_empty() {
                    None
                } else {
                    Some(0)
                };
                state.selected_path = None;
                state.pending_move = None;
                state.pending_delete = None;
            }
            TreeAction::AddChild { path, node } => {
                add_child(&mut state.root, &path, node);
            }
            TreeAction::ReplaceNode { path, node } => {
                replace_node(&mut state.root, &path, node);
            }
            TreeAction::Rename { path, label } => {
                rename_node(&mut state.root, &path, label);
            }
            TreeAction::SetSelected { path } => {
                state.selected_path = Some(path);
            }
            TreeAction::RequestDelete { path, label } => {
                state.pending_delete = Some(PendingDelete { path, label });
            }
            TreeAction::ClearPendingDelete => {
                state.pending_delete = None;
            }
            TreeAction::RemoveNode { path } => {
                remove_node(&mut state.root, &path);
                state.selected_path = None;
                state.pending_delete = None;
            }
            TreeAction::RequestMove { path, label } => {
                state.pending_move = Some(PendingMove { path, label });
            }
            TreeAction::ClearPendingMove => {
                state.pending_move = None;
            }
            TreeAction::MoveNode { from, to } => {
                if let Some(new_path) = move_node(&mut state.root, &from, &to) {
                    state.selected_path = Some(new_path);
                }
                state.pending_move = None;
            }
        }
        Rc::new(state)
    }
}

fn set_expanded(node: &mut TreeNode, path: &[usize], open: bool) {
    if path.is_empty() {
        node.expanded = open;
        return;
    }

    let Some((first, rest)) = path.split_first() else {
        return;
    };

    if let Some(child) = node.children.get_mut(*first) {
        set_expanded(child, rest, open);
    }
}

fn add_child(node: &mut TreeNode, path: &[usize], child: TreeNode) {
    if path.is_empty() {
        node.children.push(child);
        node.expanded = true;
        return;
    }

    let Some((first, rest)) = path.split_first() else {
        return;
    };

    if let Some(target) = node.children.get_mut(*first) {
        if rest.is_empty() {
            target.children.push(child);
            target.expanded = true;
        } else {
            add_child(target, rest, child);
        }
    }
}

fn rename_node(node: &mut TreeNode, path: &[usize], label: String) {
    if path.is_empty() {
        node.label = label;
        return;
    }

    let Some((first, rest)) = path.split_first() else {
        return;
    };

    if let Some(target) = node.children.get_mut(*first) {
        rename_node(target, rest, label);
    }
}

fn replace_node(node: &mut TreeNode, path: &[usize], replacement: TreeNode) {
    if path.is_empty() {
        return;
    }

    let Some((first, rest)) = path.split_first() else {
        return;
    };

    if rest.is_empty() {
        if let Some(child) = node.children.get_mut(*first) {
            *child = replacement;
        }
        return;
    }

    if let Some(target) = node.children.get_mut(*first) {
        replace_node(target, rest, replacement);
    }
}

fn remove_node(node: &mut TreeNode, path: &[usize]) {
    if path.is_empty() {
        return;
    }

    let _ = remove_node_at(node, path);
}

fn remove_node_at(node: &mut TreeNode, path: &[usize]) -> Option<TreeNode> {
    if path.is_empty() {
        return None;
    }

    let Some((first, rest)) = path.split_first() else {
        return None;
    };

    if rest.is_empty() {
        if *first < node.children.len() {
            return Some(node.children.remove(*first));
        }
        return None;
    }

    let target = node.children.get_mut(*first)?;
    remove_node_at(target, rest)
}

fn move_node(root: &mut TreeNode, from: &[usize], to: &[usize]) -> Option<Vec<usize>> {
    if from.is_empty() {
        return None;
    }
    if is_prefix_path(from, to) {
        return None;
    }

    let node = remove_node_at(root, from)?;
    let target = node_at_path_mut(root, to)?;
    target.children.push(node);
    target.expanded = true;
    let mut new_path = to.to_vec();
    new_path.push(target.children.len().saturating_sub(1));
    Some(new_path)
}

fn node_at_path_mut<'a>(root: &'a mut TreeNode, path: &[usize]) -> Option<&'a mut TreeNode> {
    let mut current = root;
    for index in path {
        current = current.children.get_mut(*index)?;
    }
    Some(current)
}

fn is_prefix_path(prefix: &[usize], path: &[usize]) -> bool {
    if prefix.len() > path.len() {
        return false;
    }
    prefix
        .iter()
        .zip(path.iter())
        .all(|(a, b)| a == b)
}

fn adjust_selected_server(
    selected: Option<usize>,
    removed_index: usize,
    remaining: usize,
) -> Option<usize> {
    let Some(selected) = selected else {
        return if remaining > 0 { Some(0) } else { None };
    };
    if selected == removed_index {
        return if remaining > 0 { Some(0) } else { None };
    }
    if selected > removed_index {
        return Some(selected.saturating_sub(1));
    }
    if selected < remaining {
        Some(selected)
    } else if remaining > 0 {
        Some(remaining - 1)
    } else {
        None
    }
}

fn tab_label_from_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('/') {
        let path = trimmed.split('?').next().unwrap_or("/");
        return Some(normalize_path(path));
    }

    if let Ok(url) = Url::parse(trimmed) {
        if matches!(url.scheme(), "http" | "https") {
            return Some(normalize_path(url.path()));
        }
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return None;
    }

    let candidate = format!("https://{trimmed}");
    Url::parse(&candidate)
        .ok()
        .map(|url| normalize_path(url.path()))
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}
