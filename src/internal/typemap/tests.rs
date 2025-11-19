use super::CloneAny;
use super::*;

#[derive(Clone, Debug, PartialEq)]
struct A(i32);
#[derive(Clone, Debug, PartialEq)]
struct B(i32);
#[derive(Clone, Debug, PartialEq)]
struct C(i32);
#[derive(Clone, Debug, PartialEq)]
struct D(i32);
#[derive(Clone, Debug, PartialEq)]
struct E(i32);
#[derive(Clone, Debug, PartialEq)]
struct F(i32);
#[derive(Clone, Debug, PartialEq)]
struct J(i32);

#[test]
fn test_default() {
    let map: TypeMap = Default::default();
    assert_eq!(map.len(), 0);
}

#[test]
fn test_expected_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_debug<T: std::fmt::Debug>() {}

    assert_send::<TypeMap<dyn Any + Send>>();
    assert_send::<TypeMap<dyn Any + Send + Sync>>();
    assert_sync::<TypeMap<dyn Any + Send + Sync>>();
    assert_debug::<TypeMap<dyn Any>>();
    assert_debug::<TypeMap<dyn Any + Send>>();
    assert_debug::<TypeMap<dyn Any + Send + Sync>>();

    assert_send::<TypeMap<dyn CloneAny + Send>>();
    assert_send::<TypeMap<dyn CloneAny + Send + Sync>>();
    assert_sync::<TypeMap<dyn CloneAny + Send + Sync>>();
    assert_debug::<TypeMap<dyn CloneAny>>();
    assert_debug::<TypeMap<dyn CloneAny + Send>>();
    assert_debug::<TypeMap<dyn CloneAny + Send + Sync>>();
    assert_clone::<TypeMap<dyn CloneAny + Send>>();
    assert_clone::<TypeMap<dyn CloneAny + Send + Sync>>();
    assert_clone::<TypeMap<dyn CloneAny + Send + Sync>>();
}

#[test]
fn test_variants() {
    /* dyn Any (+ variants) */

    let mut tm: TypeMap<dyn Any> = TypeMap::new();
    {
        assert_eq!(tm.insert(A(10)), None);
        assert_eq!(tm.insert(B(20)), None);
        assert_eq!(tm.insert(C(30)), None);
        assert_eq!(tm.insert(D(40)), None);
        assert_eq!(tm.insert(E(50)), None);
        assert_eq!(tm.insert(F(60)), None);

        // Existing key (insert)
        match tm.entry::<A>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                assert_eq!(view.get(), &A(10));
                assert_eq!(view.insert(A(100)), A(10));
            }
        }
        assert_eq!(tm.get::<A>().unwrap(), &A(100));
        assert_eq!(tm.len(), 6);

        // Existing key (update)
        match tm.entry::<B>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                let v = view.get_mut();
                let new_v = B(v.0 * 10);
                *v = new_v;
            }
        }
        assert_eq!(tm.get::<B>().unwrap(), &B(200));
        assert_eq!(tm.len(), 6);

        // Existing key (remove)
        match tm.entry::<C>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(view) => {
                assert_eq!(view.remove(), C(30));
            }
        }
        assert_eq!(tm.get::<C>(), None);
        assert_eq!(tm.len(), 5);

        // Inexistent key (insert)
        match tm.entry::<J>() {
            Entry::Occupied(_) => unreachable!(),
            Entry::Vacant(view) => {
                assert_eq!(*view.insert(J(1000)), J(1000));
            }
        }
        assert_eq!(tm.get::<J>().unwrap(), &J(1000));
        assert_eq!(tm.len(), 6);

        // Entry.or_insert on existing key
        tm.entry::<B>().or_insert(B(71)).0 += 1;
        assert_eq!(tm.get::<B>().unwrap(), &B(201));
        assert_eq!(tm.len(), 6);

        // Entry.or_insert on nonexisting key
        tm.entry::<C>().or_insert(C(300)).0 += 1;
        assert_eq!(tm.get::<C>().unwrap(), &C(301));
        assert_eq!(tm.len(), 7);
    }

    /* dyn CloneAny (+ variants) */

    let mut tm: TypeMap<dyn CloneAny> = TypeMap::new();
    {
        assert_eq!(tm.insert(A(10)), None);
        assert_eq!(tm.insert(B(20)), None);
        assert_eq!(tm.insert(C(30)), None);
        assert_eq!(tm.insert(D(40)), None);
        assert_eq!(tm.insert(E(50)), None);
        assert_eq!(tm.insert(F(60)), None);

        // Existing key (insert)
        match tm.entry::<A>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                assert_eq!(view.get(), &A(10));
                assert_eq!(view.insert(A(100)), A(10));
            }
        }
        assert_eq!(tm.get::<A>().unwrap(), &A(100));
        assert_eq!(tm.len(), 6);

        // Existing key (update)
        match tm.entry::<B>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                let v = view.get_mut();
                let new_v = B(v.0 * 10);
                *v = new_v;
            }
        }
        assert_eq!(tm.get::<B>().unwrap(), &B(200));
        assert_eq!(tm.len(), 6);

        // Existing key (remove)
        match tm.entry::<C>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(view) => {
                assert_eq!(view.remove(), C(30));
            }
        }
        assert_eq!(tm.get::<C>(), None);
        assert_eq!(tm.len(), 5);

        // Inexistent key (insert)
        match tm.entry::<J>() {
            Entry::Occupied(_) => unreachable!(),
            Entry::Vacant(view) => {
                assert_eq!(*view.insert(J(1000)), J(1000));
            }
        }
        assert_eq!(tm.get::<J>().unwrap(), &J(1000));
        assert_eq!(tm.len(), 6);

        // Entry.or_insert on existing key
        tm.entry::<B>().or_insert(B(71)).0 += 1;
        assert_eq!(tm.get::<B>().unwrap(), &B(201));
        assert_eq!(tm.len(), 6);

        // Entry.or_insert on nonexisting key
        tm.entry::<C>().or_insert(C(300)).0 += 1;
        assert_eq!(tm.get::<C>().unwrap(), &C(301));
        assert_eq!(tm.len(), 7);
    }
}

#[test]
fn test_clone() {
    let mut tm: TypeMap<dyn CloneAny> = TypeMap::new();
    let _ = tm.insert(A(1));
    let _ = tm.insert(B(2));
    /* No C */
    let _ = tm.insert(D(3));
    let _ = tm.insert(E(4));
    let _ = tm.insert(F(5));
    let _ = tm.insert(J(6));
    let tm2 = tm.clone();

    assert_eq!(tm2.len(), 6);
    assert_eq!(tm2.get::<A>(), Some(&A(1)));
    assert_eq!(tm2.get::<B>(), Some(&B(2)));
    assert_eq!(tm2.get::<C>(), None);
    assert_eq!(tm2.get::<D>(), Some(&D(3)));
    assert_eq!(tm2.get::<E>(), Some(&E(4)));
    assert_eq!(tm2.get::<F>(), Some(&F(5)));
    assert_eq!(tm2.get::<J>(), Some(&J(6)));
}
