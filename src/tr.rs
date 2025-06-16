use crate::common::il8n::{self, Il8nLocale};
use serde::ser::{self, SerializeStruct};

macro_rules! tr_il8n {
    ($name: ident, { $($code: ident => $text: literal),* }) => {
        pub fn $name(code: Il8nLocale) -> &'static str {
            let find_lang = |c| match c {
                $(Il8nLocale::$code => Some($text),)*
                #[allow(unreachable_patterns)]
                _ => None,
            };
            find_lang(code).unwrap_or_else(|| find_lang(FALLBACK_IL8N).expect("Invalid language code"))
        }
    }
}

macro_rules! tr_il8n_fast_fmt {
    ($name: ident, ($($prop: ident : $t: ty),*), { $($code: ident => ($text: literal, $($arg:ident),*) ),* } ) => {
        pub fn $name(code: Il8nLocale, $($prop: $t,)* ) -> impl FastWritable {
            struct __LangFmt {
                code: Il8nLocale,
                $($prop: $t,)*
            }
            impl FastWritable for __LangFmt {
                fn write_into<W: core::fmt::Write + ?Sized>(
                    &self,
                    dest: &mut W,
                    _values: &dyn askama::Values,
                ) -> askama::Result<()> {
                    let mut write_lang = |c| match c {
                        $(Il8nLocale::$code => Some(dest.write_fmt(format_args!($text, $(self.$arg,)* ))),)*
                        #[allow(unreachable_patterns)]
                        _ => None,
                    };
                    write_lang(self.code).unwrap_or_else(|| {
                        write_lang(FALLBACK_IL8N).expect("Invalid language code");
                        Ok(())
                    })?;
                    Ok(())
                }
            }
            return __LangFmt { code, $($prop,)* };
        }
    };
}

macro_rules! tr_il8n_fmt {
    ($name: ident, ($($prop: ident : $t: ty),*), { $($code: ident => ($text: literal, $($arg:ident),*) ),* } ) => {
        pub fn $name(code: Il8nLocale, $($prop: $t,)* ) -> String {
            let find_lang = |c| match c {
                $(Il8nLocale::$code => Some(format!($text, $($arg,)* )),)*
                #[allow(unreachable_patterns)]
                _ => None,
            };
            find_lang(code).unwrap_or_else(|| find_lang(FALLBACK_IL8N).expect("Invalid language code"))
        }
    };
}

pub const FALLBACK_IL8N: Il8nLocale = Il8nLocale::Eng;
pub const DEFAULT_IL8N: Il8nLocale  = Il8nLocale::Eng;
pub const SUPPORTED_IL8N: &[Il8nLocale ] = &[Il8nLocale::Eng];



pub struct Localization {
    code: Il8nLocale,
}

impl ser::Serialize for Localization {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut state = serializer.serialize_struct("Localization", 0)?;
        state.end()
    }
}

impl Localization {
    pub fn new(code: Il8nLocale) -> Self {
        Localization { code }
    }
}

