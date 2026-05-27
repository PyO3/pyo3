/// Polyfill of `cfg_select` for MSRV < 1.95
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
        #[cfg($cfg)]
        {
            $final_arm
        }

        #[cfg(not(any($($clauses,)* $cfg)))]
        compile_error!(
            "cfg_select! requires a final `_ => expr` arm to be used as a fallback when no other cfg matches"
        );
    };

    (
        @parsed($($clauses:meta),*)
        $cfg:meta => $arm:expr, $($rest:tt)*
    ) => {
        #[cfg($cfg)]
        {
            $arm
        }


        cfg_select! {
            @parsed($($clauses,)* $cfg)
            $($rest)*
        }
    };

    (
        @parsed($($clauses:meta),*)
        $cfg:meta => $arm:block $($rest:tt)*
    ) => {
        #[cfg($cfg)]
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
