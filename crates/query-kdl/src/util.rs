macro_rules! hashmap {
    ($($k:expr => $v:expr$(,)?)*) => {{
        let mut hashmap = std::collections::hash_map::HashMap::new();
        $(
            hashmap.insert($k, $v);
        )*
        hashmap
    }};
}
pub(crate) use hashmap;
