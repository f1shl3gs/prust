static KEYWORDS: [&str; 75] = [
    "abstract", "alignof", "as", "become", "bool", "box", "Box", "break", "const", "continue",
    "crate", "Cow", "Default", "do", "else", "enum", "Err", "extern", "f32", "f64", "false",
    "final", "fn", "for", "HashMap", "i32", "i64", "if", "impl", "in", "let", "loop", "macro",
    "match", "mod", "move", "mut", "None", "offsetof", "Ok", "Option", "override", "priv", "pub",
    "pure", "ref", "Result", "Reader", "return", "self", "Self", "sizeof", "Some", "static", "str",
    "String", "struct", "super", "trait", "true", "type", "typeof", "u8", "u32", "u64", "unsafe",
    "unsized", "use", "Vec", "virtual", "where", "while", "Write", "Writer", "yield",
];

pub fn sanitize_variant<N: AsRef<str>, S: AsRef<str>>(name: N, variant: S) -> String {
    let name = upper_camel(name.as_ref());
    let variant = upper_camel(variant.as_ref());

    let mut variant = variant
        .strip_prefix(&name)
        .map(|s| upper_camel(s))
        .unwrap_or(variant);

    if KEYWORDS.contains(&variant.as_str()) {
        variant.push('_');
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
