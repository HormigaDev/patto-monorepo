mod es;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
    PtBr,
}

impl Lang {
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Self::En,
            "pt" | "pt-br" | "pt_br" => Self::PtBr,
            "es" | "es-es" | "es-mx" | "es-ar" | "es-co" | "es-419" => Self::Es,
            _ => Self::Es,
        }
    }
}

pub fn text(locale: Lang, key: &str) -> String {
    translate(locale, key).to_string()
}

pub fn message(locale: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let mut template = translate(locale, key).to_string();

    for (name, value) in args {
        template = template.replace(&format!("{{{name}}}"), value);
    }

    template
}

fn translate(locale: Lang, key: &str) -> &'static str {
    match locale {
        Lang::Es => es::translate(key),
        _ => es::translate(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_supported_locale_aliases() {
        assert_eq!(Lang::parse("es-MX"), Lang::Es);
        assert_eq!(Lang::parse("en-us"), Lang::En);
        assert_eq!(Lang::parse("pt_BR"), Lang::PtBr);
    }

    #[test]
    fn parse_falls_back_to_spanish_for_unknown_locale() {
        assert_eq!(Lang::parse("auto"), Lang::Es);
        assert_eq!(Lang::parse("fr"), Lang::Es);
    }

    #[test]
    fn message_replaces_named_placeholders() {
        let rendered = message(
            Lang::Es,
            "cli.lint.completed",
            &[("errors", "1"), ("warnings", "2"), ("infos", "3")],
        );

        assert_eq!(
            rendered,
            "patto-core lint completed: 1 errores, 2 advertencias, 3 infos."
        );
    }

    #[test]
    fn text_returns_empty_string_for_unknown_keys() {
        assert_eq!(text(Lang::Es, "missing.key"), "");
    }
}
