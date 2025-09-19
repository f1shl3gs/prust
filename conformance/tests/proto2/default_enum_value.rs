mod prust {
    #![allow(dead_code)]

    include!("prust/default_enum_value.rs");
}

mod prost {
    include!("prost/default_enum_value.rs");
}

#[test]
fn default() {
    use prust::*;

    let msg = Test::default();

    assert_eq!(msg.privacy_level_1, PrivacyLevel::One);
    assert_eq!(msg.privacy_level_3, PrivacyLevel::PrivacyLevelThree);
    assert_eq!(msg.privacy_level_4, PrivacyLevel::PrivacyLevelFour);
}

#[test]
fn try_from_i32() {
    use prust::*;

    assert_eq!(Ok(PrivacyLevel::One), PrivacyLevel::try_from(1));
    assert_eq!(Ok(PrivacyLevel::Two), PrivacyLevel::try_from(2));
    assert_eq!(
        Ok(PrivacyLevel::PrivacyLevelThree),
        PrivacyLevel::try_from(3)
    );
    assert_eq!(
        Ok(PrivacyLevel::PrivacyLevelFour),
        PrivacyLevel::try_from(4)
    );
}

/*
#[test]
fn fuzz() {
    use ::arbitrary::Arbitrary;
    use ::prost::Message;
    use prust::{Deserialize, Serialize};

    let mut unstructured = arbitrary::Unstructured::new(b"foo bar baz");

    for _ in 0..1000 {
        let orig = <prust::Test>::arbitrary(&mut unstructured).unwrap();
        let len = orig.encoded_len();
        let mut buf = vec![0u8; len];
        orig.encode(&mut buf).unwrap();
        let out = <prust::Test>::decode(&buf).unwrap();
        assert_eq!(orig, out);

        let _prost = <prost::Test>::decode(buf.as_slice()).unwrap();
    }
}
*/
