use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::{Param, TabAction, TabState};
use crate::utils::{path_params_from_url, url_from_path_params};

#[derive(Properties, Clone, PartialEq)]
pub struct PathTableProps {
    pub tab_index: usize,
    pub url: String,
    pub path_params: Vec<Param>,
}

#[function_component(PathTable)]
pub fn path_table(props: &PathTableProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let url = props.url.clone();
    let path_params = props.path_params.clone();
    let synced_params = path_params_from_url(&url, &path_params);

    let update_path_params = {
        let tab_state = tab_state.clone();
        let url = url.clone();
        move |next_params: Vec<Param>| {
            let next_url = url_from_path_params(&url, &next_params);
            tab_state.dispatch(TabAction::UpdateUrl { index, url: next_url });
            tab_state.dispatch(TabAction::UpdatePathParams {
                index,
                path_params: next_params,
            });
        }
    };

    let on_toggle = {
        let update_path_params = update_path_params.clone();
        let path_params = synced_params.clone();
        Callback::from(move |(row_index, enabled): (usize, bool)| {
            let mut next_params = path_params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.enable = enabled;
            }
            update_path_params(next_params);
        })
    };

    let on_key_change = {
        let update_path_params = update_path_params.clone();
        let path_params = synced_params.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_params = path_params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.key = value;
            }
            update_path_params(next_params);
        })
    };

    let on_value_change = {
        let update_path_params = update_path_params.clone();
        let path_params = synced_params.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_params = path_params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.value = value;
            }
            update_path_params(next_params);
        })
    };

    let on_remove = {
        let update_path_params = update_path_params.clone();
        let path_params = synced_params.clone();
        Callback::from(move |row_index: usize| {
            if path_params.len() <= 1 {
                return;
            }
            let mut next_params = path_params.clone();
            if row_index < next_params.len() {
                next_params.remove(row_index);
            }
            update_path_params(next_params);
        })
    };

    let on_add = {
        let update_path_params = update_path_params.clone();
        let path_params = synced_params.clone();
        Callback::from(move |_| {
            let mut next_params = path_params.clone();
            next_params.push(Param {
                enable: true,
                key: String::new(),
                value: String::new(),
            });
            update_path_params(next_params);
        })
    };

    {
        let tab_state = tab_state.clone();
        let synced_params = synced_params.clone();
        let original_params = path_params.clone();
        use_effect_with((synced_params.clone(), original_params), move |(synced, original)| {
            if *synced != *original {
                tab_state.dispatch(TabAction::UpdatePathParams {
                    index,
                    path_params: synced.clone(),
                });
            }
            || ()
        });
    }

    html! {
        <div class="table-wrap">
            <h2 class="table-title">{ "Path Params" }</h2>
            <table>
                <thead>
                    <tr>
                        <th>{ "" }</th>
                        <th>{ "KEY" }</th>
                        <th>{ "VALUE" }</th>
                        <th>{ "REMOVE" }</th>
                    </tr>
                </thead>
                <tbody>
                    { for synced_params.iter().enumerate().map(|(row_index, param)| {
                        let on_toggle = {
                            let on_toggle = on_toggle.clone();
                            Callback::from(move |event: Event| {
                                let checked = event_target_checked(&event);
                                on_toggle.emit((row_index, checked));
                            })
                        };
                        let on_key_change = {
                            let on_key_change = on_key_change.clone();
                            Callback::from(move |event: InputEvent| {
                                let value = event_target_value(&event);
                                on_key_change.emit((row_index, value));
                            })
                        };
                        let on_value_change = {
                            let on_value_change = on_value_change.clone();
                            Callback::from(move |event: InputEvent| {
                                let value = event_target_value(&event);
                                on_value_change.emit((row_index, value));
                            })
                        };
                        let on_remove_click = {
                            let on_remove = on_remove.clone();
                            Callback::from(move |_| on_remove.emit(row_index))
                        };
                        html! {
                            <tr>
                                <td>
                                    <input type="checkbox" checked={param.enable} onchange={on_toggle} />
                                </td>
                                <td>
                                    <input type="text" value={param.key.clone()} oninput={on_key_change} />
                                </td>
                                <td>
                                    <input type="text" value={param.value.clone()} oninput={on_value_change} />
                                </td>
                                <td>
                                    {
                                        if synced_params.len() <= 1 {
                                            html! {}
                                        } else {
                                            html! { <button class="button ghost" onclick={on_remove_click}>{ "X" }</button> }
                                        }
                                    }
                                </td>
                            </tr>
                        }
                    }) }
                </tbody>
                <tfoot>
                    <tr>
                        <td class="table-add-cell" colspan="4">
                            <button class="button ghost table-add" onclick={on_add}>{ "+" }</button>
                        </td>
                    </tr>
                </tfoot>
            </table>
        </div>
    }
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn event_target_checked(event: &Event) -> bool {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.checked())
        .unwrap_or(false)
}
