// loan_machine_web/src/components/document_input_test.rs

use leptos::prelude::*;
use crate::components::cpf_cnpj::doc_input_snipet::DocumentInput;
use loan_machine_models::requests::DocKind;

// ── Helper ────────────────────────────────────────────────

fn render_to_string<V: IntoView>(f: impl FnOnce() -> V) -> String {
    let owner = Owner::new();
    let html = owner.with(|| f().to_html());
    drop(owner);
    html
}

fn render_cpf() -> String {
    render_to_string(|| {
        let (k, set_k) = signal(DocKind::Cpf);
        let (d, set_d) = signal(String::new());
        view! { <DocumentInput
            doc_kind=k set_doc_kind=set_k
            document=d set_document=set_d
        /> }
    })
}

fn render_cnpj() -> String {
    render_to_string(|| {
        let (k, set_k) = signal(DocKind::Cnpj);
        let (d, set_d) = signal(String::new());
        view! { <DocumentInput
            doc_kind=k set_doc_kind=set_k
            document=d set_document=set_d
        /> }
    })
}

// ── SSR output ───────────────────────────────────────────

#[test]
fn renders_both_kind_toggles() {
    let html = render_cpf();
    assert!(html.contains(">CPF<"));
    assert!(html.contains(">CNPJ<"));
}

#[test]
fn cpf_toggle_is_active_by_default_in_cpf_mode() {
    let html = render_cpf();
    // The CPF button should carry toggle-btn-active, the CNPJ one shouldn't.
    // We check ordering by splitting on CNPJ — the CPF active class must
    // appear before that split point.
    let (before_cnpj, after_cnpj) = html.split_once(">CNPJ<").unwrap();
    assert!(before_cnpj.contains("toggle-btn-active"),
        "CPF button missing active class:\n{before_cnpj}");
    // And CNPJ's own button (rendered AT >CNPJ<) shouldn't be active.
    // Grab a window after the CNPJ text up to the next </button>.
    let cnpj_btn_tail = &after_cnpj[..after_cnpj.find("</button>").unwrap_or(0)];
    assert!(!cnpj_btn_tail.contains("toggle-btn-active"),
        "CNPJ button shouldn't be active in CPF mode:\n{cnpj_btn_tail}");
}

#[test]
fn cnpj_mode_renders_cnpj_label() {
    let html = render_cnpj();
    // DocKind::Cnpj.label() — whatever it returns must appear in the form.
    let label = DocKind::Cnpj.label();
    assert!(html.contains(label),
        "expected CNPJ label `{label}` in:\n{html}");
}

#[test]
fn cpf_mode_renders_cpf_hint() {
    assert!(render_cpf().contains("11 dígitos"));
}

#[test]
fn cnpj_mode_renders_cnpj_hint() {
    assert!(render_cnpj().contains("14 dígitos"));
}

#[test]
fn maxlength_matches_kind() {
    let cpf  = render_cpf();
    let cnpj = render_cnpj();
    // The component renders maxlength=N where N is DocKind::max_input_len().
    let cpf_max  = DocKind::Cpf.max_input_len().to_string();
    let cnpj_max = DocKind::Cnpj.max_input_len().to_string();
    assert!(cpf.contains(&format!(r#"maxlength="{cpf_max}""#)),
        "expected maxlength={cpf_max} in CPF render:\n{cpf}");
    assert!(cnpj.contains(&format!(r#"maxlength="{cnpj_max}""#)),
        "expected maxlength={cnpj_max} in CNPJ render:\n{cnpj}");
}

#[test]
fn renders_no_error_by_default() {
    let html = render_cpf();
    assert!(!html.contains("form-error"));
}

#[test]
fn renders_error_when_provided() {
    let html = render_to_string(|| {
        let (k, set_k)   = signal(DocKind::Cpf);
        let (d, set_d)   = signal(String::new());
        let (err, _)     = signal("CPF inválido".to_string());
        view! { <DocumentInput
            doc_kind=k set_doc_kind=set_k
            document=d set_document=set_d
            error=err
        /> }
    });
    assert!(html.contains("form-error"));
    assert!(html.contains("CPF inválido"));
}