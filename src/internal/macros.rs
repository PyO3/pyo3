/// Polyfill of `cfg_select` for MSRV < 1.95
///
/// Note that this polyfill does not work in type position, so it is not as powerful as the
/// real `cfg_select` macro. That is a limitation that PyO3 can live with for now.
#[doc(hidden)]
#[cfg(not(cfg_select))]
macro_rules! cfg_select {
    // Final arm with _ to match any non-existing clauses
    (
        @parsed($($clauses:meta),*)
        _ => $final_arm:expr $(,)?
    ) => {
        #[cfg(not(any($($clauses),+)))]
        {
            $final_arm
        }
    };

    // Final arm has a clause, handle it plus a compile error if no clauses match
    // Need to allow for trailing comma on final expression to be optional
    (
        @parsed($($clauses:meta),*)
        $cfg:meta => $final_arm:expr $(,)?
    ) => {
        #[cfg(all($cfg, not(any($($clauses),*))))]
        {
            $final_arm
        }

        #[cfg(not(any($($clauses,)* $cfg)))]
        compile_error!(
            "cfg_select! requires a final `_ => expr` arm to be used as a fallback when no other cfg matches"
        );
    };

    // Non-terminating expression arm requires trailing comma
    (
        @parsed($($clauses:meta),*)
        $cfg:meta => $arm:expr, $($rest:tt)*
    ) => {
        #[cfg(all($cfg, not(any($($clauses),*))))]
        {
            $arm
        }

        cfg_select! {
            @parsed($($clauses,)* $cfg)
            $($rest)*
        }
    };

    // Non-terminating block doesn't require trailing comma
    (
        @parsed($($clauses:meta),*)
        $cfg:meta => $arm:block $($rest:tt)*
    ) => {
        #[cfg(all($cfg, not(any($($clauses),*))))]
        $arm

        cfg_select! {
            @parsed($($clauses,)* $cfg)
            $($rest)*
        }
    };

    (
        $cfg:meta => $($rest:tt)*
    ) => {
        {
            cfg_select! {
                @parsed()
                $cfg => $($rest)*
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(not(cfg_select))]
    fn test_cfg_select_polyfill_short_circuit() {
        cfg_select! {
            all() => {},
            all() => {
                unreachable!("the first arm should be selected, so this arm should not be evaluated");
            }
        }
    }
}
