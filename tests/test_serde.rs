#[cfg(feature = "serde")]
mod test_serde {
    use pyo3::prelude::*;

    use serde::{Deserialize, Serialize};

    #[pyclass]
    #[derive(Debug, Serialize, Deserialize)]
    struct Group {
        name: String,
    }

    #[pyclass]
    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        username: String,
        group: Option<Py<Group>>,
        friends: Vec<Py<User>>,
    }

    #[test]
    fn test_serialize() {
        let friend1 = User {
            username: "friend 1".into(),
            group: None,
            friends: vec![],
        };
        let friend2 = User {
            username: "friend 2".into(),
            group: None,
            friends: vec![],
        };

        let user = Python::attach(|py| {
            let py_friend1 = Py::new(py, friend1).expect("failed to create friend 1");
            let py_friend2 = Py::new(py, friend2).expect("failed to create friend 2");

            let friends = vec![py_friend1, py_friend2];
            let py_group = Py::new(
                py,
                Group {
                    name: "group name".into(),
                },
            )
            .unwrap();

            User {
                username: "danya".into(),
                group: Some(py_group),
                friends,
            }
        });

        let serialized = serde_json::to_string(&user).expect("failed to serialize");
        assert_eq!(
            serialized,
            r#"{"username":"danya","group":{"name":"group name"},"friends":[{"username":"friend 1","group":null,"friends":[]},{"username":"friend 2","group":null,"friends":[]}]}"#
        );
    }

    #[test]
    fn test_deserialize() {
        let serialized = r#"{"username": "danya", "friends":
        [{"username": "friend", "group": {"name": "danya's friends"}, "friends": []}]}"#;
        let user: User = serde_json::from_str(serialized).expect("failed to deserialize");

        assert_eq!(user.username, "danya");
        assert!(user.group.is_none());
        assert_eq!(user.friends.len(), 1usize);
        let friend = user.friends.first().unwrap();

        Python::attach(|py| {
            assert_eq!(friend.borrow(py).username, "friend");
            assert_eq!(
                friend.borrow(py).group.as_ref().unwrap().borrow(py).name,
                "danya's friends"
            )
        });
    }
}
