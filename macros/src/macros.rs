macro_rules! ensure {
    ($cond:expr, $msg:expr) => {
        ensure!($cond, $msg, ::proc_macro2::Span::call_site())
    };
    ($cond:expr, $msg:expr, $span:expr) => {
        if !$cond {
            return Err(::syn::Error::new($span, $msg));
        }
    };
}

pub(crate) use ensure;
