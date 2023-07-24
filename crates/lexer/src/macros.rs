macro_rules! identify {
    (
        $self:expr,
        [$($name:ident),*],
        [$($rep: expr => ($token_ty: ident, $val: expr)),*],
         $($matcher:pat => $result:expr),*) => {
        paste::paste! {
            match $self.buffer.as_str() {
                $(stringify!([<$name:lower>]) => $self.add_token(TokenType::$name, None),)*
                $($rep => $self.add_token($token_ty, Some($val)),)*
                $($matcher => $result)*
            }
        }
    };
}

pub(crate) use identify;
