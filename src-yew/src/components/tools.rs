use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use ed25519_dalek::Signer;
use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use hmac::{Hmac, Mac};
use js_sys::{decode_uri_component, encode_uri_component};
use rand::rngs::OsRng;
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use wasm_bindgen::JsCast;
use yew::prelude::*;

type HmacSha1 = Hmac<Sha1>;

#[function_component(ToolsPage)]
pub fn tools_page() -> Html {
    let location_hash = use_state(current_hash);
    let base64_input = use_state(String::new);
    let base64_output = use_state(String::new);
    let base64_error = use_state(String::new);

    let url_input = use_state(String::new);
    let url_output = use_state(String::new);
    let url_error = use_state(String::new);

    let totp_secret = use_state(String::new);
    let totp_code = use_state(String::new);
    let totp_remaining = use_state(|| 0u64);
    let totp_error = use_state(String::new);

    let hash_input = use_state(String::new);
    let hash_algorithm = use_state(|| "sha256".to_string());
    let hash_output = use_state(String::new);
    let hash_error = use_state(String::new);

    let sym_key = use_state(String::new);
    let sym_nonce = use_state(String::new);
    let sym_input = use_state(String::new);
    let sym_output = use_state(String::new);
    let sym_error = use_state(String::new);

    let rsa_public_key = use_state(String::new);
    let rsa_private_key = use_state(String::new);
    let rsa_input = use_state(String::new);
    let rsa_output = use_state(String::new);
    let rsa_error = use_state(String::new);

    let ed_message = use_state(String::new);
    let ed_private_key = use_state(String::new);
    let ed_public_key = use_state(String::new);
    let ed_signature = use_state(String::new);
    let ed_error = use_state(String::new);
    let ed_verify_status = use_state(String::new);

    let on_tab_tools = Callback::from(move |_| {
        set_hash("#tools");
    });
    let on_tab_crypto = Callback::from(move |_| {
        set_hash("#crypto");
    });

    let on_base64_input = {
        let base64_input = base64_input.clone();
        Callback::from(move |event: InputEvent| {
            base64_input.set(event_target_value(&event));
        })
    };

    let on_base64_encode = {
        let base64_input = base64_input.clone();
        let base64_output = base64_output.clone();
        let base64_error = base64_error.clone();
        Callback::from(move |_| {
            base64_error.set(String::new());
            let encoded = STANDARD.encode(base64_input.as_bytes());
            base64_output.set(encoded);
        })
    };

    let on_base64_decode = {
        let base64_input = base64_input.clone();
        let base64_output = base64_output.clone();
        let base64_error = base64_error.clone();
        Callback::from(move |_| {
            let input = base64_input.trim();
            match STANDARD.decode(input) {
                Ok(bytes) => {
                    base64_error.set(String::new());
                    base64_output.set(String::from_utf8_lossy(&bytes).to_string());
                }
                Err(_) => {
                    base64_output.set(String::new());
                    base64_error.set("Invalid base64 input.".to_string());
                }
            }
        })
    };

    let on_url_input = {
        let url_input = url_input.clone();
        Callback::from(move |event: InputEvent| {
            url_input.set(event_target_value(&event));
        })
    };

    let on_url_encode = {
        let url_input = url_input.clone();
        let url_output = url_output.clone();
        let url_error = url_error.clone();
        Callback::from(move |_| {
            url_error.set(String::new());
            url_output.set(
                encode_uri_component(&url_input)
                    .as_string()
                    .unwrap_or_default(),
            );
        })
    };

    let on_url_decode = {
        let url_input = url_input.clone();
        let url_output = url_output.clone();
        let url_error = url_error.clone();
        Callback::from(move |_| match decode_uri_component(&url_input) {
            Ok(decoded) => {
                url_error.set(String::new());
                url_output.set(decoded.as_string().unwrap_or_default());
            }
            Err(_) => {
                url_output.set(String::new());
                url_error.set("Invalid URL encoding.".to_string());
            }
        })
    };

    let on_totp_secret = {
        let totp_secret = totp_secret.clone();
        Callback::from(move |event: InputEvent| {
            totp_secret.set(event_target_value(&event));
        })
    };

    let on_hash_input = {
        let hash_input = hash_input.clone();
        Callback::from(move |event: InputEvent| {
            hash_input.set(event_target_value(&event));
        })
    };
    let on_hash_algorithm = {
        let hash_algorithm = hash_algorithm.clone();
        Callback::from(move |event: Event| {
            hash_algorithm.set(event_target_select(&event));
        })
    };
    let on_hash_compute = {
        let hash_input = hash_input.clone();
        let hash_algorithm = hash_algorithm.clone();
        let hash_output = hash_output.clone();
        let hash_error = hash_error.clone();
        Callback::from(move |_| {
            match compute_hash(&hash_input, &hash_algorithm) {
                Ok(value) => {
                    hash_output.set(value);
                    hash_error.set(String::new());
                }
                Err(err) => {
                    hash_output.set(String::new());
                    hash_error.set(err);
                }
            }
        })
    };

    {
        let totp_secret = totp_secret.clone();
        let totp_code = totp_code.clone();
        let totp_remaining = totp_remaining.clone();
        let totp_error = totp_error.clone();
        use_effect_with(totp_secret.clone(), move |secret| {
            let secret_value = secret.trim().to_string();
            if secret_value.is_empty() {
                totp_code.set(String::new());
                totp_remaining.set(0);
                totp_error.set(String::new());
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            let secret_for_tick = secret_value.clone();
            let totp_code = totp_code.clone();
            let totp_remaining = totp_remaining.clone();
            let totp_error = totp_error.clone();
            let tick = move |secret: &str,
                             totp_code: &UseStateHandle<String>,
                             totp_remaining: &UseStateHandle<u64>,
                             totp_error: &UseStateHandle<String>| {
                match generate_totp(secret) {
                    Ok((code, remaining)) => {
                        totp_code.set(code);
                        totp_remaining.set(remaining);
                        totp_error.set(String::new());
                    }
                    Err(err) => {
                        totp_code.set(String::new());
                        totp_remaining.set(0);
                        totp_error.set(err);
                    }
                }
            };

            tick(&secret_for_tick, &totp_code, &totp_remaining, &totp_error);

            let interval = Interval::new(1000, move || {
                tick(&secret_for_tick, &totp_code, &totp_remaining, &totp_error);
            });

            Box::new(move || drop(interval)) as Box<dyn FnOnce()>
        });
    }

    {
        let location_hash = location_hash.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("window not available");
            let listener = EventListener::new(&window, "hashchange", move |_| {
                location_hash.set(current_hash());
            });
            Box::new(move || drop(listener)) as Box<dyn FnOnce()>
        });
    }

    let active_tab = tools_tab_from_hash(location_hash.as_str());

    html! {
        <div class="tools-page">
            <div class="tab-list tools-tabs">
                <button
                    class={tab_class(active_tab == "tools")}
                    onclick={on_tab_tools}
                    type="button"
                >
                    { "Tools" }
                </button>
                <button
                    class={tab_class(active_tab == "crypto")}
                    onclick={on_tab_crypto}
                    type="button"
                >
                    { "Crypto" }
                </button>
            </div>
            <div class="tools-tab-content">
                {
                    if active_tab == "tools" {
                        html! {
                            <div class="tools-grid">
                                <div class="tools-section">
                                    <h2>{ "Base64" }</h2>
                                    <textarea
                                        class="tools-input"
                                        placeholder="Paste text or base64 here"
                                        value={(*base64_input).clone()}
                                        oninput={on_base64_input}
                                    />
                                    <div class="tools-actions">
                                        <button class="button" onclick={on_base64_encode}>{ "Encode" }</button>
                                        <button class="button secondary" onclick={on_base64_decode}>{ "Decode" }</button>
                                    </div>
                                    {
                                        if !base64_error.is_empty() {
                                            html! { <p class="tools-error">{ (*base64_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <textarea
                                        class="tools-output"
                                        readonly=true
                                        value={(*base64_output).clone()}
                                        placeholder="Result"
                                    />
                                </div>

                                <div class="tools-section">
                                    <h2>{ "URL Encode / Decode" }</h2>
                                    <textarea
                                        class="tools-input"
                                        placeholder="Paste text or encoded URL"
                                        value={(*url_input).clone()}
                                        oninput={on_url_input}
                                    />
                                    <div class="tools-actions">
                                        <button class="button" onclick={on_url_encode}>{ "Encode" }</button>
                                        <button class="button secondary" onclick={on_url_decode}>{ "Decode" }</button>
                                    </div>
                                    {
                                        if !url_error.is_empty() {
                                            html! { <p class="tools-error">{ (*url_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <textarea
                                        class="tools-output"
                                        readonly=true
                                        value={(*url_output).clone()}
                                        placeholder="Result"
                                    />
                                </div>

                                <div class="tools-section">
                                    <h2>{ "TOTP Generator" }</h2>
                                    <label class="tools-label">{ "Secret (Base32)" }</label>
                                    <input
                                        class="tools-input-line"
                                        placeholder="JBSWY3DPEHPK3PXP"
                                        value={(*totp_secret).clone()}
                                        oninput={on_totp_secret}
                                    />
                                    <div class="tools-totp-row">
                                        <div class="tools-totp-code">
                                            {
                                                if totp_code.is_empty() {
                                                    "---".to_string()
                                                } else {
                                                    (*totp_code).clone()
                                                }
                                            }
                                        </div>
                                        <div class="tools-totp-meta">
                                            { format!("Expires in {}s", *totp_remaining) }
                                        </div>
                                    </div>
                                    {
                                        if !totp_error.is_empty() {
                                            html! { <p class="tools-error">{ (*totp_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>

                                <div class="tools-section">
                                    <h2>{ "Hash" }</h2>
                                    <textarea
                                        class="tools-input"
                                        placeholder="Text to hash"
                                        value={(*hash_input).clone()}
                                        oninput={on_hash_input.clone()}
                                    />
                                    <div class="tools-row">
                                        <select class="tools-select" onchange={on_hash_algorithm.clone()} value={(*hash_algorithm).clone()}>
                                            <option value="sha1">{ "SHA-1" }</option>
                                            <option value="sha256">{ "SHA-256" }</option>
                                            <option value="sha512">{ "SHA-512" }</option>
                                            <option value="md5">{ "MD5" }</option>
                                        </select>
                                        <button class="button" onclick={on_hash_compute.clone()}>{ "Compute" }</button>
                                    </div>
                                    {
                                        if !hash_error.is_empty() {
                                            html! { <p class="tools-error">{ (*hash_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <textarea
                                        class="tools-output"
                                        readonly=true
                                        value={(*hash_output).clone()}
                                        placeholder="Hash output"
                                    />
                                </div>
                            </div>
                        }
                    } else {
                        let on_sym_key = {
                            let sym_key = sym_key.clone();
                            Callback::from(move |event: InputEvent| {
                                sym_key.set(event_target_value(&event));
                            })
                        };
                        let on_sym_nonce = {
                            let sym_nonce = sym_nonce.clone();
                            Callback::from(move |event: InputEvent| {
                                sym_nonce.set(event_target_value(&event));
                            })
                        };
                        let on_sym_input = {
                            let sym_input = sym_input.clone();
                            Callback::from(move |event: InputEvent| {
                                sym_input.set(event_target_value(&event));
                            })
                        };
                        let on_sym_encrypt = {
                            let sym_key = sym_key.clone();
                            let sym_nonce = sym_nonce.clone();
                            let sym_input = sym_input.clone();
                            let sym_output = sym_output.clone();
                            let sym_error = sym_error.clone();
                            Callback::from(move |_| {
                                match encrypt_symmetric(&sym_key, &sym_nonce, &sym_input) {
                                    Ok(value) => {
                                        sym_output.set(value);
                                        sym_error.set(String::new());
                                    }
                                    Err(err) => {
                                        sym_output.set(String::new());
                                        sym_error.set(err);
                                    }
                                }
                            })
                        };
                        let on_sym_decrypt = {
                            let sym_key = sym_key.clone();
                            let sym_nonce = sym_nonce.clone();
                            let sym_input = sym_input.clone();
                            let sym_output = sym_output.clone();
                            let sym_error = sym_error.clone();
                            Callback::from(move |_| {
                                match decrypt_symmetric(&sym_key, &sym_nonce, &sym_input) {
                                    Ok(value) => {
                                        sym_output.set(value);
                                        sym_error.set(String::new());
                                    }
                                    Err(err) => {
                                        sym_output.set(String::new());
                                        sym_error.set(err);
                                    }
                                }
                            })
                        };

                        let on_rsa_public_key = {
                            let rsa_public_key = rsa_public_key.clone();
                            Callback::from(move |event: InputEvent| {
                                rsa_public_key.set(event_target_value(&event));
                            })
                        };
                        let on_rsa_private_key = {
                            let rsa_private_key = rsa_private_key.clone();
                            Callback::from(move |event: InputEvent| {
                                rsa_private_key.set(event_target_value(&event));
                            })
                        };
                        let on_rsa_input = {
                            let rsa_input = rsa_input.clone();
                            Callback::from(move |event: InputEvent| {
                                rsa_input.set(event_target_value(&event));
                            })
                        };
                        let on_rsa_encrypt = {
                            let rsa_public_key = rsa_public_key.clone();
                            let rsa_input = rsa_input.clone();
                            let rsa_output = rsa_output.clone();
                            let rsa_error = rsa_error.clone();
                            Callback::from(move |_| {
                                match rsa_encrypt(&rsa_public_key, &rsa_input) {
                                    Ok(value) => {
                                        rsa_output.set(value);
                                        rsa_error.set(String::new());
                                    }
                                    Err(err) => {
                                        rsa_output.set(String::new());
                                        rsa_error.set(err);
                                    }
                                }
                            })
                        };
                        let on_rsa_decrypt = {
                            let rsa_private_key = rsa_private_key.clone();
                            let rsa_input = rsa_input.clone();
                            let rsa_output = rsa_output.clone();
                            let rsa_error = rsa_error.clone();
                            Callback::from(move |_| {
                                match rsa_decrypt(&rsa_private_key, &rsa_input) {
                                    Ok(value) => {
                                        rsa_output.set(value);
                                        rsa_error.set(String::new());
                                    }
                                    Err(err) => {
                                        rsa_output.set(String::new());
                                        rsa_error.set(err);
                                    }
                                }
                            })
                        };

                        let on_ed_message = {
                            let ed_message = ed_message.clone();
                            Callback::from(move |event: InputEvent| {
                                ed_message.set(event_target_value(&event));
                            })
                        };
                        let on_ed_private_key = {
                            let ed_private_key = ed_private_key.clone();
                            Callback::from(move |event: InputEvent| {
                                ed_private_key.set(event_target_value(&event));
                            })
                        };
                        let on_ed_public_key = {
                            let ed_public_key = ed_public_key.clone();
                            Callback::from(move |event: InputEvent| {
                                ed_public_key.set(event_target_value(&event));
                            })
                        };
                        let on_ed_signature = {
                            let ed_signature = ed_signature.clone();
                            Callback::from(move |event: InputEvent| {
                                ed_signature.set(event_target_value(&event));
                            })
                        };
                        let on_ed_sign = {
                            let ed_message = ed_message.clone();
                            let ed_private_key = ed_private_key.clone();
                            let ed_signature = ed_signature.clone();
                            let ed_error = ed_error.clone();
                            let ed_verify_status = ed_verify_status.clone();
                            Callback::from(move |_| {
                                match ed_sign_message(&ed_message, &ed_private_key) {
                                    Ok(signature) => {
                                        ed_signature.set(signature);
                                        ed_error.set(String::new());
                                        ed_verify_status.set(String::new());
                                    }
                                    Err(err) => {
                                        ed_signature.set(String::new());
                                        ed_error.set(err);
                                        ed_verify_status.set(String::new());
                                    }
                                }
                            })
                        };
                        let on_ed_verify = {
                            let ed_message = ed_message.clone();
                            let ed_public_key = ed_public_key.clone();
                            let ed_signature = ed_signature.clone();
                            let ed_error = ed_error.clone();
                            let ed_verify_status = ed_verify_status.clone();
                            Callback::from(move |_| {
                                match ed_verify_message(&ed_message, &ed_public_key, &ed_signature) {
                                    Ok(valid) => {
                                        ed_error.set(String::new());
                                        ed_verify_status.set(if valid { "Signature valid." } else { "Signature invalid." }.to_string());
                                    }
                                    Err(err) => {
                                        ed_error.set(err);
                                        ed_verify_status.set(String::new());
                                    }
                                }
                            })
                        };

                        html! {
                            <div class="tools-grid">
                                <div class="tools-section">
                                    <h2>{ "Symmetric (AES-256-GCM)" }</h2>
                                    <label class="tools-label">{ "Key (hex or base64, 32 bytes)" }</label>
                                    <input
                                        class="tools-input-line"
                                        value={(*sym_key).clone()}
                                        oninput={on_sym_key}
                                    />
                                    <label class="tools-label">{ "Nonce (hex or base64, 12 bytes)" }</label>
                                    <input
                                        class="tools-input-line"
                                        value={(*sym_nonce).clone()}
                                        oninput={on_sym_nonce}
                                    />
                                    <label class="tools-label">{ "Input (plain text for encrypt, base64 for decrypt)" }</label>
                                    <textarea
                                        class="tools-input"
                                        value={(*sym_input).clone()}
                                        oninput={on_sym_input}
                                    />
                                    <div class="tools-actions">
                                        <button class="button" onclick={on_sym_encrypt}>{ "Encrypt" }</button>
                                        <button class="button secondary" onclick={on_sym_decrypt}>{ "Decrypt" }</button>
                                    </div>
                                    {
                                        if !sym_error.is_empty() {
                                            html! { <p class="tools-error">{ (*sym_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <textarea
                                        class="tools-output"
                                        readonly=true
                                        value={(*sym_output).clone()}
                                        placeholder="Result"
                                    />
                                </div>

                                <div class="tools-section">
                                    <h2>{ "RSA Encrypt / Decrypt" }</h2>
                                    <label class="tools-label">{ "Public key (PEM)" }</label>
                                    <textarea
                                        class="tools-input"
                                        placeholder="-----BEGIN PUBLIC KEY-----"
                                        value={(*rsa_public_key).clone()}
                                        oninput={on_rsa_public_key}
                                    />
                                    <label class="tools-label">{ "Private key (PEM)" }</label>
                                    <textarea
                                        class="tools-input"
                                        placeholder="-----BEGIN PRIVATE KEY-----"
                                        value={(*rsa_private_key).clone()}
                                        oninput={on_rsa_private_key}
                                    />
                                    <label class="tools-label">{ "Input (plain text for encrypt, base64 for decrypt)" }</label>
                                    <textarea
                                        class="tools-input"
                                        value={(*rsa_input).clone()}
                                        oninput={on_rsa_input}
                                    />
                                    <div class="tools-actions">
                                        <button class="button" onclick={on_rsa_encrypt}>{ "Encrypt" }</button>
                                        <button class="button secondary" onclick={on_rsa_decrypt}>{ "Decrypt" }</button>
                                    </div>
                                    {
                                        if !rsa_error.is_empty() {
                                            html! { <p class="tools-error">{ (*rsa_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <textarea
                                        class="tools-output"
                                        readonly=true
                                        value={(*rsa_output).clone()}
                                        placeholder="Result"
                                    />
                                </div>

                                <div class="tools-section">
                                    <h2>{ "Ed25519 Sign / Verify" }</h2>
                                    <label class="tools-label">{ "Message" }</label>
                                    <textarea
                                        class="tools-input"
                                        value={(*ed_message).clone()}
                                        oninput={on_ed_message}
                                    />
                                    <label class="tools-label">{ "Private key (hex or base64, 32 bytes)" }</label>
                                    <input
                                        class="tools-input-line"
                                        value={(*ed_private_key).clone()}
                                        oninput={on_ed_private_key}
                                    />
                                    <label class="tools-label">{ "Public key (hex or base64, 32 bytes)" }</label>
                                    <input
                                        class="tools-input-line"
                                        value={(*ed_public_key).clone()}
                                        oninput={on_ed_public_key}
                                    />
                                    <label class="tools-label">{ "Signature (base64, 64 bytes)" }</label>
                                    <textarea
                                        class="tools-input"
                                        value={(*ed_signature).clone()}
                                        oninput={on_ed_signature}
                                    />
                                    <div class="tools-actions">
                                        <button class="button" onclick={on_ed_sign}>{ "Sign" }</button>
                                        <button class="button secondary" onclick={on_ed_verify}>{ "Verify" }</button>
                                    </div>
                                    {
                                        if !ed_error.is_empty() {
                                            html! { <p class="tools-error">{ (*ed_error).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if !ed_verify_status.is_empty() {
                                            html! { <p class="tools-status">{ (*ed_verify_status).clone() }</p> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            </div>
                        }
                    }
                }
            </div>
        </div>
    }
}

fn tab_class(active: bool) -> Classes {
    if active {
        classes!("tab-trigger", "active")
    } else {
        classes!("tab-trigger")
    }
}

fn tools_tab_from_hash(hash: &str) -> &'static str {
    if hash == "#crypto" {
        "crypto"
    } else {
        "tools"
    }
}

fn current_hash() -> String {
    web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default()
}

fn set_hash(value: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(value);
    }
}

fn event_target_select(event: &Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| {
            if let Ok(input) = target.clone().dyn_into::<web_sys::HtmlInputElement>() {
                return Some(input.value());
            }
            if let Ok(textarea) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                return Some(textarea.value());
            }
            None
        })
        .unwrap_or_default()
}

fn generate_totp(secret: &str) -> Result<(String, u64), String> {
    let key = decode_base32(secret).ok_or_else(|| "Invalid base32 secret.".to_string())?;
    if key.is_empty() {
        return Err("Invalid base32 secret.".to_string());
    }

    let now = (js_sys::Date::now() / 1000.0).floor() as u64;
    let step = 30u64;
    let counter = now / step;
    let remaining = step - (now % step);

    let mut mac =
        <HmacSha1 as Mac>::new_from_slice(&key).map_err(|_| "Invalid secret key.".to_string())?;
    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();
    if result.len() < 20 {
        return Err("Invalid HMAC result.".to_string());
    }
    let offset = (result[19] & 0x0f) as usize;
    let slice = &result[offset..offset + 4];
    let code = ((slice[0] as u32 & 0x7f) << 24)
        | ((slice[1] as u32) << 16)
        | ((slice[2] as u32) << 8)
        | (slice[3] as u32);
    let otp = code % 1_000_000;
    Ok((format!("{otp:06}"), remaining))
}

fn decode_base32(value: &str) -> Option<Vec<u8>> {
    let filtered: String = value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-')
        .collect();
    let normalized = filtered.to_ascii_uppercase();
    base32::decode(base32::Alphabet::RFC4648 { padding: false }, &normalized)
}

fn compute_hash(input: &str, algorithm: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let output = match algorithm {
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        }
        "md5" => {
            let digest = md5::compute(bytes);
            format!("{:x}", digest)
        }
        _ => return Err("Unsupported hash algorithm.".to_string()),
    };
    Ok(output)
}

fn rsa_encrypt(public_key_pem: &str, message: &str) -> Result<String, String> {
    let public_key = parse_rsa_public_key(public_key_pem)?;
    let mut rng = OsRng;
    let encrypted = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, message.as_bytes())
        .map_err(|err| format!("RSA encrypt failed: {err}"))?;
    Ok(STANDARD.encode(encrypted))
}

fn rsa_decrypt(private_key_pem: &str, ciphertext_base64: &str) -> Result<String, String> {
    let private_key = parse_rsa_private_key(private_key_pem)?;
    let cipher_bytes = decode_binary_input(ciphertext_base64)?;
    let decrypted = private_key
        .decrypt(Pkcs1v15Encrypt, &cipher_bytes)
        .map_err(|err| format!("RSA decrypt failed: {err}"))?;
    Ok(String::from_utf8_lossy(&decrypted).to_string())
}

fn parse_rsa_public_key(value: &str) -> Result<RsaPublicKey, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Public key is required.".to_string());
    }
    RsaPublicKey::from_public_key_pem(trimmed)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(trimmed))
        .map_err(|_| "Invalid RSA public key PEM.".to_string())
}

fn parse_rsa_private_key(value: &str) -> Result<RsaPrivateKey, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Private key is required.".to_string());
    }
    RsaPrivateKey::from_pkcs8_pem(trimmed)
        .or_else(|_| RsaPrivateKey::from_pkcs1_pem(trimmed))
        .map_err(|_| "Invalid RSA private key PEM.".to_string())
}

fn ed_sign_message(message: &str, private_key_input: &str) -> Result<String, String> {
    let key_bytes = decode_binary_input(private_key_input)?;
    let key_array: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Private key must be 32 bytes.".to_string())?;
    let signing_key = SigningKey::from_bytes(&key_array);
    let signature: Signature = signing_key.sign(message.as_bytes());
    Ok(STANDARD.encode(signature.to_bytes()))
}

fn ed_verify_message(
    message: &str,
    public_key_input: &str,
    signature_input: &str,
) -> Result<bool, String> {
    let public_key_bytes = decode_binary_input(public_key_input)?;
    let public_key_array: [u8; 32] = public_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Public key must be 32 bytes.".to_string())?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_key_array).map_err(|_| "Invalid public key.".to_string())?;

    let signature_bytes = decode_binary_input(signature_input)?;
    let signature_array: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Signature must be 64 bytes.".to_string())?;
    let signature = Signature::from_bytes(&signature_array);
    Ok(verifying_key.verify(message.as_bytes(), &signature).is_ok())
}

fn encrypt_symmetric(key_input: &str, nonce_input: &str, plaintext: &str) -> Result<String, String> {
    let key_bytes = decode_binary_input(key_input)?;
    let nonce_bytes = decode_binary_input(nonce_input)?;
    let key_array: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Key must be 32 bytes.".to_string())?;
    let nonce_array: [u8; 12] = nonce_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Nonce must be 12 bytes.".to_string())?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_array));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_array), plaintext.as_bytes())
        .map_err(|_| "Encryption failed.".to_string())?;
    Ok(STANDARD.encode(ciphertext))
}

fn decrypt_symmetric(
    key_input: &str,
    nonce_input: &str,
    ciphertext_input: &str,
) -> Result<String, String> {
    let key_bytes = decode_binary_input(key_input)?;
    let nonce_bytes = decode_binary_input(nonce_input)?;
    let cipher_bytes = decode_binary_input(ciphertext_input)?;
    let key_array: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Key must be 32 bytes.".to_string())?;
    let nonce_array: [u8; 12] = nonce_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Nonce must be 12 bytes.".to_string())?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_array));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_array), cipher_bytes.as_ref())
        .map_err(|_| "Decryption failed.".to_string())?;
    Ok(String::from_utf8_lossy(&plaintext).to_string())
}

fn decode_binary_input(value: &str) -> Result<Vec<u8>, String> {
    let mut trimmed: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
    if trimmed.is_empty() {
        return Err("Value is required.".to_string());
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        trimmed = trimmed[2..].to_string();
    }
    let is_hex = trimmed
        .chars()
        .all(|ch| ch.is_ascii_hexdigit());
    if is_hex && trimmed.len() % 2 == 0 {
        return hex::decode(trimmed).map_err(|_| "Invalid hex value.".to_string());
    }
    STANDARD
        .decode(trimmed)
        .map_err(|_| "Invalid base64 value.".to_string())
}
