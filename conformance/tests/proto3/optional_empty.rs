mod prust {
    #![allow(unused_variables)]

    include!("prust/optional_empty.rs");

    #[test]
    fn size() {
        let empty = Empty::default();
        assert_eq!(empty.encoded_len(), 0);
        let msg = Foo::default();
        assert_eq!(msg.encoded_len(), 0);
        let msg = Foo {
            empty: Some(Empty {}),
        };
        assert_eq!(msg.encoded_len(), 2);
    }
}

mod prost {
    include!("prost/optional_empty.rs");

    #[test]
    fn size() {
        use ::prost::Message;

        let empty = Empty::default();
        assert_eq!(empty.encoded_len(), 0);
        let msg = Foo::default();
        assert_eq!(msg.encoded_len(), 0);
        let msg = Foo {
            empty: Some(Empty {}),
        };
        assert_eq!(msg.encoded_len(), 2);
    }
}
