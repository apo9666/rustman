use gloo::events::EventListener;
use yew::prelude::*;

use wasm_bindgen::JsCast;

use crate::state::{ApiKeyLocation, OAuth2Flow, OAuthScope, ServerAuth, TreeAction, TreeState};

#[derive(Properties, Clone, PartialEq)]
pub struct SideProps {
    pub on_add_server: Callback<()>,
    pub on_add_tag: Callback<()>,
}

#[function_component(Side)]
pub fn side(props: &SideProps) -> Html {
    let tree_state = use_context::<UseReducerHandle<TreeState>>();

    let Some(tree_state) = tree_state else {
        return html! {};
    };

    {
        let tree_state = tree_state.clone();
        let servers_len = tree_state.servers.len();
        let selected = tree_state.selected_server;
        use_effect_with((servers_len, selected), move |(len, selected)| {
            if *len > 0 && selected.is_none() {
                tree_state.dispatch(TreeAction::SetSelectedServer { index: 0 });
            }
            || ()
        });
    }

    let on_cancel_move = {
        let tree_state = tree_state.clone();
        Callback::from(move |_| {
            tree_state.dispatch(TreeAction::ClearPendingMove);
        })
    };

    let servers = tree_state.servers.clone();
    let selected_server = tree_state
        .selected_server
        .filter(|index| *index < servers.len())
        .or_else(|| if servers.is_empty() { None } else { Some(0) });

    let on_server_change = {
        let tree_state = tree_state.clone();
        Callback::from(move |event: Event| {
            let value = select_value(&event);
            if let Ok(index) = value.parse::<usize>() {
                tree_state.dispatch(TreeAction::SetSelectedServer { index });
            }
        })
    };

    let menu_open = use_state(|| false);
    let menu_ref = use_node_ref();
    let pending_remove_server = use_state(|| None::<usize>);
    let auth_dialog_open = use_state(|| false);
    let auth_form = use_state(AuthForm::default);
    let auth_server_index = use_state(|| None::<usize>);
    let on_menu_toggle = {
        let menu_open = menu_open.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(!*menu_open);
        })
    };

    {
        let menu_open = menu_open.clone();
        let menu_ref = menu_ref.clone();
        use_effect_with(menu_open.clone(), move |is_open| {
            if !**is_open {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let listener = EventListener::new(&document, "click", move |event| {
                let target = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
                let menu_node = menu_ref.cast::<web_sys::Node>();
                if let (Some(menu_node), Some(target)) = (menu_node, target) {
                    if menu_node.contains(Some(&target)) {
                        return;
                    }
                }
                menu_open.set(false);
            });
            Box::new(move || drop(listener)) as Box<dyn FnOnce()>
        });
    }

    let on_add_server = {
        let menu_open = menu_open.clone();
        let on_add_server = props.on_add_server.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            on_add_server.emit(());
        })
    };

    let on_add_tag = {
        let menu_open = menu_open.clone();
        let on_add_tag = props.on_add_tag.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            on_add_tag.emit(());
        })
    };

    let on_remove_server = {
        let tree_state = tree_state.clone();
        let menu_open = menu_open.clone();
        let selected_server = selected_server.clone();
        let pending_remove_server = pending_remove_server.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            let Some(index) = selected_server else {
                return;
            };
            if tree_state.servers.get(index).is_some() {
                pending_remove_server.set(Some(index));
            }
        })
    };

    let on_edit_auth = {
        let menu_open = menu_open.clone();
        let auth_dialog_open = auth_dialog_open.clone();
        let auth_form = auth_form.clone();
        let auth_server_index = auth_server_index.clone();
        let selected_server = selected_server.clone();
        let servers = servers.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            let Some(index) = selected_server else {
                return;
            };
            let Some(server) = servers.get(index) else {
                return;
            };
            auth_form.set(AuthForm::from_auth(&server.auth));
            auth_server_index.set(Some(index));
            auth_dialog_open.set(true);
        })
    };

    let on_auth_save = {
        let tree_state = tree_state.clone();
        let auth_form = auth_form.clone();
        let auth_dialog_open = auth_dialog_open.clone();
        let auth_server_index = auth_server_index.clone();
        Callback::from(move |_| {
            if let Some(index) = (*auth_server_index).clone() {
                let auth = (*auth_form).to_auth();
                tree_state.dispatch(TreeAction::UpdateServerAuth { index, auth });
            }
            auth_dialog_open.set(false);
            auth_server_index.set(None);
        })
    };

    let on_auth_cancel = {
        let auth_dialog_open = auth_dialog_open.clone();
        let auth_server_index = auth_server_index.clone();
        Callback::from(move |_| {
            auth_dialog_open.set(false);
            auth_server_index.set(None);
        })
    };

    html! {
        <>
        <nav class="tree-panel">
            <div class="tree-header">
                <div class="server-select-wrap">
                    <select
                        class="server-select"
                        onchange={on_server_change}
                        value={selected_server.map(|index| index.to_string()).unwrap_or_default()}
                        disabled={servers.is_empty()}
                    >
                        {
                            if servers.is_empty() {
                                html! { <option value="">{ "No servers" }</option> }
                            } else {
                                html! { for servers.iter().enumerate().map(|(index, server)| {
                                    html! { <option value={index.to_string()}>{ server.url.clone() }</option> }
                                }) }
                            }
                        }
                    </select>
                    <span class="select-chevron"></span>
                </div>
                <div class="tree-menu-wrap" ref={menu_ref}>
                    <button type="button" class="tree-row-menu" title="Servers" onclick={on_menu_toggle}>{ "⋯" }</button>
                    {
                        if *menu_open {
                            html! {
                                <div class="tree-menu">
                                    <button type="button" class="tree-menu-item" onclick={on_add_server.clone()}>
                                        { "Add server" }
                                    </button>
                                    <button type="button" class="tree-menu-item" onclick={on_add_tag.clone()}>
                                        { "Add tag" }
                                    </button>
                                    <button type="button" class="tree-menu-item" onclick={on_edit_auth.clone()}>
                                        { "Auth" }
                                    </button>
                                    <button type="button" class="tree-menu-item danger" onclick={on_remove_server.clone()}>
                                        { "Remove server" }
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>
            {
                if let Some(pending_move) = tree_state.pending_move.as_ref() {
                    html! {
                        <div class="tree-banner">
                            <div class="tree-banner-text">
                                <span class="tree-banner-title">{ "Move:" }</span>
                                <span class="tree-banner-label">{ pending_move.label.clone() }</span>
                                <span class="muted">{ "Select destination tag." }</span>
                            </div>
                            <button class="tree-banner-cancel" type="button" onclick={on_cancel_move.clone()}>
                                { "Cancel" }
                            </button>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            <div class="tree">
                {
                    if tree_state.root.children.is_empty() {
                        html! { <div class="tree-empty">{ "Add a tag to get started." }</div> }
                    } else {
                        html! { <crate::components::tree::directory::TreeDirectory node={tree_state.root.clone()} path={vec![]} /> }
                    }
                }
            </div>
        </nav>
        {
            if let Some(index) = (*pending_remove_server).clone() {
                let label = tree_state
                    .servers
                    .get(index)
                    .map(|server| server.url.clone())
                    .unwrap_or_default();
                let on_confirm = {
                    let tree_state = tree_state.clone();
                    let pending_remove_server = pending_remove_server.clone();
                    Callback::from(move |_| {
                        tree_state.dispatch(TreeAction::RemoveServer { index });
                        pending_remove_server.set(None);
                    })
                };
                let on_cancel = {
                    let pending_remove_server = pending_remove_server.clone();
                    Callback::from(move |_| {
                        pending_remove_server.set(None);
                    })
                };
                let on_confirm_click =
                    Callback::from(move |_event: MouseEvent| on_confirm.emit(()));
                let on_cancel_click =
                    Callback::from(move |_event: MouseEvent| on_cancel.emit(()));

                html! {
                    <div class="modal-backdrop">
                        <div class="modal">
                            <h2 class="modal-title">{ "Remove server" }</h2>
                            <p class="modal-text">{ format!("Remove \"{}\"?", label) }</p>
                            <div class="modal-actions">
                                <button class="button secondary" onclick={on_cancel_click}>{ "Cancel" }</button>
                                <button class="button danger" onclick={on_confirm_click}>{ "Remove" }</button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }
        }
        {
            if *auth_dialog_open {
                let auth_form_value = (*auth_form).clone();
                let on_kind_change = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: Event| {
                        let value = select_value(&event);
                        let mut next = (*auth_form).clone();
                        next.kind = value;
                        auth_form.set(next);
                    })
                };

                let on_api_key_name = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.api_key_name = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_api_key_in = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: Event| {
                        let mut next = (*auth_form).clone();
                        next.api_key_in = select_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_api_key_value = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.api_key_value = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_basic_user = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.basic_username = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_basic_pass = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.basic_password = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_bearer_token = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.bearer_token = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_bearer_format = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.bearer_format = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_flow = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: Event| {
                        let mut next = (*auth_form).clone();
                        next.oauth_flow = select_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_auth_url = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oauth_auth_url = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_token_url = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oauth_token_url = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_refresh_url = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oauth_refresh_url = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_scopes = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oauth_scopes = textarea_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oauth_token = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oauth_access_token = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oidc_url = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oidc_url = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_oidc_token = {
                    let auth_form = auth_form.clone();
                    Callback::from(move |event: InputEvent| {
                        let mut next = (*auth_form).clone();
                        next.oidc_access_token = input_value(&event);
                        auth_form.set(next);
                    })
                };

                let on_confirm_click = {
                    let on_auth_save = on_auth_save.clone();
                    Callback::from(move |_event: MouseEvent| on_auth_save.emit(()))
                };

                let on_cancel_click = {
                    let on_auth_cancel = on_auth_cancel.clone();
                    Callback::from(move |_event: MouseEvent| on_auth_cancel.emit(()))
                };

                html! {
                    <div class="modal-backdrop">
                        <div class="modal auth-modal">
                            <h2 class="modal-title">{ "Server auth" }</h2>
                            <label class="modal-label">{ "Type" }</label>
                            <select class="modal-input" onchange={on_kind_change} value={auth_form_value.kind.clone()}>
                                <option value="none">{ "None" }</option>
                                <option value="apiKey">{ "API key" }</option>
                                <option value="httpBasic">{ "HTTP Basic" }</option>
                                <option value="httpBearer">{ "HTTP Bearer" }</option>
                                <option value="oauth2">{ "OAuth2" }</option>
                                <option value="openIdConnect">{ "OpenID Connect" }</option>
                            </select>

                            {
                                match auth_form_value.kind.as_str() {
                                    "apiKey" => html! {
                                        <>
                                            <label class="modal-label">{ "Name" }</label>
                                            <input class="modal-input" value={auth_form_value.api_key_name.clone()} oninput={on_api_key_name} />
                                            <label class="modal-label">{ "Location" }</label>
                                            <select class="modal-input" onchange={on_api_key_in} value={auth_form_value.api_key_in.clone()}>
                                                <option value="header">{ "Header" }</option>
                                                <option value="query">{ "Query" }</option>
                                                <option value="cookie">{ "Cookie" }</option>
                                            </select>
                                            <label class="modal-label">{ "Value" }</label>
                                            <input class="modal-input" value={auth_form_value.api_key_value.clone()} oninput={on_api_key_value} />
                                        </>
                                    },
                                    "httpBasic" => html! {
                                        <>
                                            <label class="modal-label">{ "Username" }</label>
                                            <input class="modal-input" value={auth_form_value.basic_username.clone()} oninput={on_basic_user} />
                                            <label class="modal-label">{ "Password" }</label>
                                            <input class="modal-input" type="password" value={auth_form_value.basic_password.clone()} oninput={on_basic_pass} />
                                        </>
                                    },
                                    "httpBearer" => html! {
                                        <>
                                            <label class="modal-label">{ "Token" }</label>
                                            <input class="modal-input" type="password" value={auth_form_value.bearer_token.clone()} oninput={on_bearer_token} />
                                            <label class="modal-label">{ "Bearer format" }</label>
                                            <input class="modal-input" value={auth_form_value.bearer_format.clone()} oninput={on_bearer_format} />
                                        </>
                                    },
                                    "oauth2" => html! {
                                        <>
                                            <label class="modal-label">{ "Flow" }</label>
                                            <select class="modal-input" onchange={on_oauth_flow} value={auth_form_value.oauth_flow.clone()}>
                                                <option value="authorizationCode">{ "Authorization code" }</option>
                                                <option value="implicit">{ "Implicit" }</option>
                                                <option value="password">{ "Password" }</option>
                                                <option value="clientCredentials">{ "Client credentials" }</option>
                                            </select>
                                            <label class="modal-label">{ "Authorization URL" }</label>
                                            <input class="modal-input" value={auth_form_value.oauth_auth_url.clone()} oninput={on_oauth_auth_url} />
                                            <label class="modal-label">{ "Token URL" }</label>
                                            <input class="modal-input" value={auth_form_value.oauth_token_url.clone()} oninput={on_oauth_token_url} />
                                            <label class="modal-label">{ "Refresh URL" }</label>
                                            <input class="modal-input" value={auth_form_value.oauth_refresh_url.clone()} oninput={on_oauth_refresh_url} />
                                            <label class="modal-label">{ "Scopes (one per line: scope=description)" }</label>
                                            <textarea class="modal-textarea" value={auth_form_value.oauth_scopes.clone()} oninput={on_oauth_scopes} />
                                            <label class="modal-label">{ "Access token" }</label>
                                            <input class="modal-input" type="password" value={auth_form_value.oauth_access_token.clone()} oninput={on_oauth_token} />
                                        </>
                                    },
                                    "openIdConnect" => html! {
                                        <>
                                            <label class="modal-label">{ "OpenID Connect URL" }</label>
                                            <input class="modal-input" value={auth_form_value.oidc_url.clone()} oninput={on_oidc_url} />
                                            <label class="modal-label">{ "Access token" }</label>
                                            <input class="modal-input" type="password" value={auth_form_value.oidc_access_token.clone()} oninput={on_oidc_token} />
                                        </>
                                    },
                                    _ => html! {},
                                }
                            }

                            <div class="modal-actions">
                                <button class="button secondary" onclick={on_cancel_click}>{ "Cancel" }</button>
                                <button class="button" onclick={on_confirm_click}>{ "Save" }</button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }
        }
        </>
    }
}

fn select_value(event: &Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn input_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn textarea_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

#[derive(Clone, PartialEq)]
struct AuthForm {
    kind: String,
    api_key_name: String,
    api_key_in: String,
    api_key_value: String,
    basic_username: String,
    basic_password: String,
    bearer_token: String,
    bearer_format: String,
    oauth_flow: String,
    oauth_auth_url: String,
    oauth_token_url: String,
    oauth_refresh_url: String,
    oauth_scopes: String,
    oauth_access_token: String,
    oidc_url: String,
    oidc_access_token: String,
}

impl Default for AuthForm {
    fn default() -> Self {
        Self {
            kind: "none".to_string(),
            api_key_name: String::new(),
            api_key_in: "header".to_string(),
            api_key_value: String::new(),
            basic_username: String::new(),
            basic_password: String::new(),
            bearer_token: String::new(),
            bearer_format: "Bearer".to_string(),
            oauth_flow: "authorizationCode".to_string(),
            oauth_auth_url: String::new(),
            oauth_token_url: String::new(),
            oauth_refresh_url: String::new(),
            oauth_scopes: String::new(),
            oauth_access_token: String::new(),
            oidc_url: String::new(),
            oidc_access_token: String::new(),
        }
    }
}

impl AuthForm {
    fn from_auth(auth: &ServerAuth) -> Self {
        match auth {
            ServerAuth::None => Self::default(),
            ServerAuth::ApiKey {
                name,
                location,
                value,
            } => Self {
                kind: "apiKey".to_string(),
                api_key_name: name.clone(),
                api_key_in: location.as_str().to_string(),
                api_key_value: value.clone(),
                ..Self::default()
            },
            ServerAuth::HttpBasic { username, password } => Self {
                kind: "httpBasic".to_string(),
                basic_username: username.clone(),
                basic_password: password.clone(),
                ..Self::default()
            },
            ServerAuth::HttpBearer {
                token,
                bearer_format,
            } => Self {
                kind: "httpBearer".to_string(),
                bearer_token: token.clone(),
                bearer_format: if bearer_format.trim().is_empty() {
                    "Bearer".to_string()
                } else {
                    bearer_format.clone()
                },
                ..Self::default()
            },
            ServerAuth::OAuth2 {
                flow,
                auth_url,
                token_url,
                refresh_url,
                scopes,
                access_token,
            } => Self {
                kind: "oauth2".to_string(),
                oauth_flow: flow.as_str().to_string(),
                oauth_auth_url: auth_url.clone(),
                oauth_token_url: token_url.clone(),
                oauth_refresh_url: refresh_url.clone(),
                oauth_scopes: scopes_to_text(scopes),
                oauth_access_token: access_token.clone(),
                ..Self::default()
            },
            ServerAuth::OpenIdConnect { url, access_token } => Self {
                kind: "openIdConnect".to_string(),
                oidc_url: url.clone(),
                oidc_access_token: access_token.clone(),
                ..Self::default()
            },
        }
    }

    fn to_auth(&self) -> ServerAuth {
        match self.kind.as_str() {
            "apiKey" => ServerAuth::ApiKey {
                name: self.api_key_name.clone(),
                location: ApiKeyLocation::from_str(&self.api_key_in)
                    .unwrap_or(ApiKeyLocation::Header),
                value: self.api_key_value.clone(),
            },
            "httpBasic" => ServerAuth::HttpBasic {
                username: self.basic_username.clone(),
                password: self.basic_password.clone(),
            },
            "httpBearer" => ServerAuth::HttpBearer {
                token: self.bearer_token.clone(),
                bearer_format: if self.bearer_format.trim().is_empty() {
                    "Bearer".to_string()
                } else {
                    self.bearer_format.clone()
                },
            },
            "oauth2" => ServerAuth::OAuth2 {
                flow: OAuth2Flow::from_str(&self.oauth_flow)
                    .unwrap_or(OAuth2Flow::AuthorizationCode),
                auth_url: self.oauth_auth_url.clone(),
                token_url: self.oauth_token_url.clone(),
                refresh_url: self.oauth_refresh_url.clone(),
                scopes: scopes_from_text(&self.oauth_scopes),
                access_token: self.oauth_access_token.clone(),
            },
            "openIdConnect" => ServerAuth::OpenIdConnect {
                url: self.oidc_url.clone(),
                access_token: self.oidc_access_token.clone(),
            },
            _ => ServerAuth::None,
        }
    }
}

fn scopes_to_text(scopes: &[OAuthScope]) -> String {
    scopes
        .iter()
        .map(|scope| {
            if scope.description.trim().is_empty() {
                scope.name.clone()
            } else {
                format!("{}={}", scope.name, scope.description)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn scopes_from_text(text: &str) -> Vec<OAuthScope> {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.splitn(2, '=');
            let name = parts.next().unwrap_or("").trim().to_string();
            let description = parts.next().unwrap_or("").trim().to_string();
            OAuthScope { name, description }
        })
        .filter(|scope| !scope.name.is_empty())
        .collect()
}
