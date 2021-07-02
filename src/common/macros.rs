#![doc(hidden)]
/// This macro is useful for forcing repeat expression - particularly optional
/// items - without actually outputting anything depending on the input.
macro_rules! blank {
    ($in:ident) => {};
}

/// Outputs true if an input is present, otherwise false.
macro_rules! true_if_present {
    ($in:ident) => {
        true
    };

    () => {
        false
    };
}

/// Outputs the first token tree if present, otherwise the second.
macro_rules! or_else {
    ($present:tt,$absent:tt) => {
        $present
    };

    (,$absent:tt) => {
        $absent
    };
}

/// Outputs the first type if present, otherwise the second.
macro_rules! or_else_type {
    ($present:ty,$default:ty) => {
        $present
    };

    ( ,$default:ty ) => {
        $default
    };
}

/// If the first token-tree is present it chooses the second one as output; if not, the third
macro_rules! choose_from_presence {
    ($in:tt $present:tt, $absent:tt) => {
        $present
    };

    ($present:tt, $absent:tt) => {
        $absent
    };
}
