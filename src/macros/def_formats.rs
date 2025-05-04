
#[macro_export]
macro_rules! def_formats {
    (
        $(
            $enum_name:ident {                  // like Audio
                $(
                    $variant:ident              // like Mp3

                    $( (ext=$ext:expr) )?       // 可选的扩展名(ext="...")

                    $( (decs=$decs:expr) )?     // 可选的描述(decs="...")
                ),*
            }
        ),*
    ) => {
        $(
            #[derive(Clone, Debug, PartialEq, EnumIter, AsRefStr,Data)]
            pub enum $enum_name {
                $(
                    $variant
                ),*
            }

            impl $enum_name {
                pub fn desc(&self) -> Option<&'static str> {
                    match self {
                        $(
                            $enum_name::$variant => {
                                def_formats!(@get_desc $( $decs )?)
                            }
                        ),*
                    }
                }

                pub fn ext(&self) -> &'static str {
                    match self {
                        $(
                            $enum_name::$variant => {
                                def_formats!(@get_ext stringify!($variant) $(, $ext )?)
                            }
                        ),*
                    }
                }
            }
        )*
    };

    // 辅助规则：desc
    (@get_desc $d:expr) => {
        Some($d)
    };
    (@get_desc) => {
        None
    };

    // 辅助规则：ext
    (@get_ext $_default:expr, $e:expr) => {
        $e
    };
    (@get_ext $default:expr) => {{
        match $default {
            name => name.to_lowercase().leak(),
        }
    }};
}