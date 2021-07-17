use dill::*;

#[test]
fn test_send_to_thread() {
    #[component]
    struct AImpl;

    impl AImpl {
        fn foo(&self) -> String {
            "foo".to_owned()
        }
    }

    let mut cat = Catalog::new();
    cat.add::<AImpl>();

    let res = std::thread::spawn(move || {
        let a = cat.get_one::<AImpl>().unwrap();
        a.foo()
    })
    .join()
    .unwrap();

    assert_eq!(res, "foo");
}

#[test]
fn test_clone_to_threads() {
    #[component]
    struct AImpl;

    impl AImpl {
        fn foo(&self) -> String {
            "foo".to_owned()
        }
    }

    let mut cat = Catalog::new();
    cat.add::<AImpl>();

    let (res1, res2) = {
        let cat_t1 = cat.clone();
        let cat_t2 = cat.clone();

        let h1 = std::thread::spawn(move || {
            let a = cat_t1.get_one::<AImpl>().unwrap();
            a.foo()
        });

        let h2 = std::thread::spawn(move || {
            let a = cat_t2.get_one::<AImpl>().unwrap();
            a.foo()
        });

        (h1.join().unwrap(), h2.join().unwrap())
    };

    assert_eq!(res1, "foo");
    assert_eq!(res2, "foo");
}
