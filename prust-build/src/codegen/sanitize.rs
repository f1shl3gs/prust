// https://doc.rust-lang.org/reference/keywords.html#reserved-keywords
static KEYWORDS: [&str; 56] = [
    // The following keywords are in all editions
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    // The following keywords were added beginning in the 2018 edition.
    "async",
    "await",
    "dyn",
    // Reserved but not used
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    // Reserved in the 2018 edition
    "try",
    // Reserved in the 2024 edition
    "gen",
    // Weak keywords, These keywords have special meaning only in certain contexts
    "macro_rules",
    "raw",
    "safe",
    "union",
];

pub fn sanitize_variant<N: AsRef<str>, S: AsRef<str>>(name: N, variant: S) -> String {
    let name = upper_camel(name.as_ref());
    let cased = upper_camel(variant.as_ref());

    let mut variant = cased
        .strip_prefix(&name)
        .map(upper_camel)
        .unwrap_or(cased.clone());

    if KEYWORDS.contains(&variant.as_str()) {
        variant.push('_');
    }

    if variant.as_bytes()[0].is_ascii_digit() {
        return cased;
    }

    variant
}

pub fn sanitize_field<S: AsRef<str>>(s: S) -> String {
    let cased = snake(s.as_ref());

    if KEYWORDS.contains(&cased.as_ref()) {
        format!("r#{}", cased)
    } else {
        cased
    }
}

#[inline]
pub fn sanitize_filepath<S: AsRef<str>>(name: S) -> String {
    let name = name.as_ref();

    name.strip_suffix(".proto")
        .unwrap_or_else(|| name)
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

pub fn sanitize_type<S: AsRef<str>>(typ: S) -> String {
    let typ = typ.as_ref();

    match typ.rsplit_once('.') {
        Some((path, typ)) => {
            let path = path.split('.').map(snake).collect::<Vec<_>>().join("::");

            format!("{}::{}", path, typ)
        }
        None => upper_camel(typ),
    }
}

pub fn sanitize_type_name<S: AsRef<str>>(typ: S) -> String {
    let typ = upper_camel(typ.as_ref());

    if typ == "Self" { typ + "_" } else { typ }
}

pub fn snake(ident: &str) -> String {
    if KEYWORDS.contains(&ident) {
        return format!("r#{}", ident);
    }

    let mut buf = String::new();
    for (index, ch) in ident.char_indices() {
        if ch.is_ascii_uppercase() {
            if index != 0 && buf.chars().last() != Some('_') {
                buf.push('_');
            }

            buf.push_str(ch.to_lowercase().to_string().as_str());
            continue;
        }

        buf.push(ch);
    }

    buf
}

pub fn upper_camel(ident: &str) -> String {
    let mut buf = String::new();

    for word in ident
        .split('_')
        .map(|word| word[..1].to_ascii_uppercase() + &word[1..])
    {
        if !word.contains(|ch: char| ch.is_ascii_lowercase()) {
            // ALL UPPER
            buf.push_str(
                (word[..1].to_ascii_uppercase() + &word[1..].to_ascii_lowercase()).as_str(),
            );
        } else {
            buf.push_str(&word);
        }
    }

    buf
}
