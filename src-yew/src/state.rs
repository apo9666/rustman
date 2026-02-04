use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

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
            url: String::new(),
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
            tabs: vec![Tab {
                label: "New Tab".to_string(),
                content: TabContent::default(),
            }],
        }
    }
}

pub enum TabAction {
    AddTab,
    OpenTab { label: String, content: TabContent },
    CloseTab(usize),
    SetActive(usize),
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
                    label: "New".to_string(),
                    content: TabContent::default(),
                });
                state.active_tab_id = state.tabs.len().saturating_sub(1);
            }
            TabAction::OpenTab { label, content } => {
                state.tabs.push(Tab { label, content });
                state.active_tab_id = state.tabs.len().saturating_sub(1);
            }
            TabAction::CloseTab(index) => {
                if index < state.tabs.len() {
                    state.tabs.remove(index);
                }
                if state.tabs.is_empty() {
                    state.tabs.push(Tab {
                        label: "New Tab".to_string(),
                        content: TabContent::default(),
                    });
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
            TabAction::UpdateMethod { index, method } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.method = method;
                }
            }
            TabAction::UpdateUrl { index, url } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.url = url;
                }
            }
            TabAction::UpdateBody { index, body } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.body = body;
                    tab.content.body_formatted = false;
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
                }
            }
            TabAction::SetHeaders { index, headers } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.headers = headers;
                }
            }
            TabAction::UpdateUrlAndParams { index, url, params } => {
                if let Some(tab) = state.tabs.get_mut(index) {
                    tab.content.url = url;
                    tab.content.params = params;
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
        }
    }
}

pub enum TreeAction {
    SetExpanded { path: Vec<usize>, open: bool },
    AddRootChild(TreeNode),
    AddChild { path: Vec<usize>, node: TreeNode },
    Rename { path: Vec<usize>, label: String },
}

impl Reducible for TreeState {
    type Action = TreeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            TreeAction::SetExpanded { path, open } => {
                set_expanded(&mut state.root, &path, open);
            }
            TreeAction::AddRootChild(node) => {
                state.root.children.push(node);
            }
            TreeAction::AddChild { path, node } => {
                add_child(&mut state.root, &path, node);
            }
            TreeAction::Rename { path, label } => {
                rename_node(&mut state.root, &path, label);
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
