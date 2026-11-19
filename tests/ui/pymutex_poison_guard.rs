// Internal definition locations may be absolute paths; only the caller diagnostics matter here.
//@normalize-stderr-test: "note: [^\n]*\n *--> [^\n]*src/sync.rs[^\n]*\n(?: *[0-9]* *[|][^\n]*\n)*" -> ""

use pyo3::sync::PyMutexTrait;

fn without_token<'a, M: PyMutexTrait<'a, ()>>(mutex: &'a M) {
    let _guard = mutex.poison_guard();
    //~^ ERROR: this method takes 1 argument but 0 arguments were supplied
}

fn private_token<'a, M: PyMutexTrait<'a, ()>>(mutex: &'a M) {
    let _guard = mutex.poison_guard(pyo3::sync::mutex_trait_sealed::Token);
    //~^ ERROR: module `mutex_trait_sealed` is private
}

fn inferred_token<'a, M: PyMutexTrait<'a, ()>>(mutex: &'a M) {
    let _guard = mutex.poison_guard(Default::default());
    //~^ ERROR: the trait bound `pyo3::sync::mutex_trait_sealed::Token: Default` is not satisfied
}

fn main() {}
