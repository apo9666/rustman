use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::{Header, TabAction, TabState};

#[derive(Properties, Clone, PartialEq)]
pub struct HeaderTableProps {
    pub tab_index: usize,
    pub headers: Vec<Header>,
}

#[function_component(HeaderTable)]
pub fn header_table(props: &HeaderTableProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let headers = props.headers.clone();

    let update_headers = {
        let tab_state = tab_state.clone();
        move |mut next_headers: Vec<Header>| {
            ensure_trailing_header(&mut next_headers);
            tab_state.dispatch(TabAction::SetHeaders {
                index,
                headers: next_headers,
            });
        }
    };

    let on_toggle = {
        let update_headers = update_headers.clone();
        let headers = headers.clone();
        Callback::from(move |(row_index, enabled): (usize, bool)| {
            let mut next_headers = headers.clone();
            if let Some(header) = next_headers.get_mut(row_index) {
                header.enable = enabled;
            }
            update_headers(next_headers);
        })
    };

    let on_key_change = {
        let update_headers = update_headers.clone();
        let headers = headers.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_headers = headers.clone();
            if let Some(header) = next_headers.get_mut(row_index) {
                header.key = value;
            }
            update_headers(next_headers);
        })
    };

    let on_value_change = {
        let update_headers = update_headers.clone();
        let headers = headers.clone();
        Callback::from(move |(row_index, value): (usize, String)| {
            let mut next_headers = headers.clone();
            if let Some(header) = next_headers.get_mut(row_index) {
                header.value = value;
            }
            update_headers(next_headers);
        })
    };

    let on_remove = {
        let update_headers = update_headers.clone();
        let headers = headers.clone();
        Callback::from(move |row_index: usize| {
            let mut next_headers = headers.clone();
            if row_index < next_headers.len() {
                next_headers.remove(row_index);
            }
            update_headers(next_headers);
        })
    };

    html! {
        <div class="table-wrap">
            <h2 class="table-title">{ "Headers" }</h2>
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
                    { for headers.iter().enumerate().map(|(row_index, header)| {
                        let is_last = row_index + 1 == headers.len();
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
                                    <input type="checkbox" checked={header.enable} onchange={on_toggle} />
                                </td>
                                <td>
                                    <input type="text" value={header.key.clone()} oninput={on_key_change} />
                                </td>
                                <td>
                                    <input type="text" value={header.value.clone()} oninput={on_value_change} />
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

fn ensure_trailing_header(headers: &mut Vec<Header>) {
    let needs_trailing = headers.last().map(|header| {
        !header.key.trim().is_empty() || !header.value.trim().is_empty() || !header.enable
    });
    if needs_trailing.unwrap_or(true) {
        headers.push(Header {
            enable: true,
            key: String::new(),
            value: String::new(),
        });
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
