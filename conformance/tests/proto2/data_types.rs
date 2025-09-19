mod prust {
    #![allow(unused_imports)]

    include!("prust/data_types.rs");
}

mod a {
    pub mod b {
        include!("prost/a.b.rs");
    }
}
mod prost {
    include!("prost/data_types.rs");
}

conformance::fuzz!(FooMessage);
