use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::{Param, TabAction, TabState};
use crate::utils::{ensure_trailing_param, url_from_params};

#[derive(Properties, Clone, PartialEq)]
pub struct ParamTableProps {
    pub tab_index: usize,
    pub url: String,
    pub params: Vec<Param>,
}

#[function_component(ParamTable)]
pub fn param_table(props: &ParamTableProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let url = props.url.clone();
    let params = props.params.clone();

    let update_params = {
        let tab_state = tab_state.clone();
        move |mut next_params: Vec<Param>| {
            ensure_trailing_param(&mut next_params);
            let next_url = url_from_params(&url, &next_params);
            tab_state.dispatch(TabAction::UpdateUrlAndParams {
                index,
                url: next_url,
                params: next_params,
            });
        }
    };

    let on_toggle = {
        let update_params = update_params.clone();
        let params = params.clone();
        Callback::from(move |(row_index, enabled): (usize, bool)| {
            let mut next_params = params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.enable = enabled;
            }
            update_params(next_params);
        })
    };

    let on_key_change = {
        let update_params = update_params.clone();
        let params = params.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_params = params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.key = value;
            }
            update_params(next_params);
        })
    };

    let on_value_change = {
        let update_params = update_params.clone();
        let params = params.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_params = params.clone();
            if let Some(param) = next_params.get_mut(row_index) {
                param.value = value;
            }
            update_params(next_params);
        })
    };

    let on_remove = {
        let update_params = update_params.clone();
        let params = params.clone();
        Callback::from(move |row_index: usize| {
            let mut next_params = params.clone();
            if row_index < next_params.len() {
                next_params.remove(row_index);
            }
            update_params(next_params);
        })
    };

    html! {
        <div class="table-wrap">
            <h2 class="table-title">{ "Query Params" }</h2>
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
                    { for params.iter().enumerate().map(|(row_index, param)| {
                        let is_last = row_index + 1 == params.len();
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
                                        if is_last {
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
