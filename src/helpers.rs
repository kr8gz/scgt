macro_rules! helper_functions {
    (
        % PREFIX $prefix:literal
        $( $name:ident: $spwn_name:literal, )*
    ) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub enum HelperFunction {
            $( $name ),*
        }
        
        impl HelperFunction {
            pub fn spwn_name(&self) -> &'static str {
                match self {
                    $( Self::$name => concat!($prefix, $spwn_name) ),*
                }
            }
        
            pub fn spwn_impl(&self) -> &'static str {
                match self {
                    $( Self::$name => include_str!(concat!("spwn/", $spwn_name, ".spwn")) ),*
                }
            }
        }
    }
}

helper_functions! {
    % PREFIX "_scgt_"
    Bool: "bool",
    Call: "call",
    Get: "get",
    Invert: "invert",
    Iter: "iter",
    Mod: "mod",
    Mul: "mul",
    Print: "print",
    Set: "set",
}
