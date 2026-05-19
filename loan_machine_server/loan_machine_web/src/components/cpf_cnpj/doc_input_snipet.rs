use leptos::prelude::*;
use loan_machine_models::requests::DocKind;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
/// CPF / CNPJ document input with kind toggle.
///
/// Internal behavior owned by this component:
///   • Clicking CPF or CNPJ switches the kind AND clears the document
///     (formatting/length rules differ between the two).
///   • Typing filters everything except digits and the visual-formatting
///     chars `.`, `-`, `/`.  The DOM input value is rewritten in place
///     if the user pasted disallowed chars, so cursor jumps aren't
///     possible.
///   • `maxlength` and `placeholder` track the current kind.
///
/// The parent passes both signal pairs so it can read final values at
/// submit time or reset them externally if needed.  All UI logic
/// (toggle clears document, input filtering) lives here — no closures
/// should ever be duplicated at the call site.
#[component]
pub fn DocumentInput(
    doc_kind:     ReadSignal<DocKind>,
    set_doc_kind: WriteSignal<DocKind>,
    document:     ReadSignal<String>,
    set_document: WriteSignal<String>,
    #[prop(optional, into)] error: Signal<String>,
) -> impl IntoView {
    let switch_to = move |kind: DocKind| {
        set_doc_kind.set(kind);
        set_document.set(String::new());
    };

    let on_document_input = move |ev: leptos::ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let raw     = input.value();
        let max_len = doc_kind.get().max_input_len();
        let cleaned: String = raw
            .chars()
            .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '/'))
            .take(max_len)
            .collect();
        if cleaned != raw {
            input.set_value(&cleaned);
        }
        set_document.set(cleaned);
    };

    view! {
        <div class="flex-col gap-3">
            <div class="doc-kind-toggle" role="tablist">
                <button
                    type="button"
                    role="tab"
                    class=move || if doc_kind.get() == DocKind::Cpf {
                        "toggle-btn toggle-btn-active"
                    } else {
                        "toggle-btn"
                    }
                    on:click=move |_| switch_to(DocKind::Cpf)
                >
                    "CPF"
                </button>
                <button
                    type="button"
                    role="tab"
                    class=move || if doc_kind.get() == DocKind::Cnpj {
                        "toggle-btn toggle-btn-active"
                    } else {
                        "toggle-btn"
                    }
                    on:click=move |_| switch_to(DocKind::Cnpj)
                >
                    "CNPJ"
                </button>
            </div>

            <div class="form-group">
                <label class="form-label">
                    {move || doc_kind.get().label()}
                </label>
                <input
                    class="form-input"
                    style=move || if !error.get().is_empty() {
                        "border-color: var(--c-red);"
                    } else {
                        ""
                    }
                    type="text"
                    inputmode="numeric"
                    autocomplete="off"
                    prop:value=document
                    placeholder=move || doc_kind.get().placeholder()
                    maxlength=move || doc_kind.get().max_input_len() as i32
                    on:input=on_document_input
                />
                <span class="form-hint">
                    {move || match doc_kind.get() {
                        DocKind::Cpf  => "11 dígitos — formatação opcional",
                        DocKind::Cnpj => "14 dígitos — formatação opcional",
                    }}
                </span>
                {move || {
                    let e = error.get();
                    if e.is_empty() {
                        ().into_any()
                    } else {
                        view! { <span class="form-error">"⚠ "{e}</span> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}