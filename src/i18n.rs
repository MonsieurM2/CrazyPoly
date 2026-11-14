//! Runtime localisation (French / English).
//!
//! The game is single-threaded, so the active language lives in one atomic
//! value that both the engine and the UI read while drawing. Every visible
//! string passes through `t` (static labels) or the `tr!` macro (messages with
//! placeholders), so switching language takes effect on the next frame.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Fr,
    En,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn lang() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == 1 {
        Lang::En
    } else {
        Lang::Fr
    }
}

pub fn set_lang(lang: Lang) {
    CURRENT.store(if lang == Lang::En { 1 } else { 0 }, Ordering::Relaxed);
}

pub fn toggle() -> Lang {
    let next = if lang() == Lang::Fr { Lang::En } else { Lang::Fr };
    set_lang(next);
    next
}

/// Short code shown on the language button ("FR" / "EN").
pub fn code() -> &'static str {
    if lang() == Lang::Fr {
        "FR"
    } else {
        "EN"
    }
}

pub fn is_fr() -> bool {
    lang() == Lang::Fr
}

/// Pick the string matching the active language.
pub fn t(en: &'static str, fr: &'static str) -> &'static str {
    if is_fr() {
        fr
    } else {
        en
    }
}

/// Fill `{}` placeholders left to right.
pub fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    let mut index = 0;
    while let Some(pos) = rest.find("{}") {
        out.push_str(&rest[..pos]);
        out.push_str(args.get(index).copied().unwrap_or(""));
        index += 1;
        rest = &rest[pos + 2..];
    }
    out.push_str(rest);
    out
}

/// `tr!("{ } buys { }", "{ } achète { }", name, price)` in the active language.
#[macro_export]
macro_rules! tr {
    ($en:expr, $fr:expr $(, $arg:expr)* $(,)?) => {{
        let __args: Vec<String> = vec![$(($arg).to_string()),*];
        let __refs: Vec<&str> = __args.iter().map(|s| s.as_str()).collect();
        $crate::i18n::fill($crate::i18n::t($en, $fr), &__refs)
    }};
}
