use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct RequestTitleProps {
    pub title: String,
    pub on_save: Callback<()>,
}

#[function_component(RequestTitle)]
pub fn request_title(props: &RequestTitleProps) -> Html {
    let on_save = {
        let on_save = props.on_save.clone();
        Callback::from(move |_| on_save.emit(()))
    };

    html! {
        <div class="request-title">
            <h1>{ props.title.clone() }</h1>
            <div class="request-actions">
                <button class="button secondary" onclick={on_save}>{ "Save" }</button>
            </div>
        </div>
    }
}
