macro_rules! map {
    (
        /// $desc:expr
        $(#[$attrs:meta])*
        pub struct $name:ident$(<$a:lifetime>)?(IndexMap<$key:ty, $value:ty, DefaultHashBuilder>);
    ) => {
        ///
        $(#[$attrs])*
        pub struct $name$(<$a>)?(IndexMap<$key, $value, DefaultHashBuilder>);

        impl<$($a,)? 'b> IntoIterator for &'b $name$(<$a>)? {
            type Item = (&'b $key, &'b $value);
            type IntoIter = Iter<'b, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl$(<$a>)? Default for $name$(<$a>)? {
            fn default() -> Self {
                Self::new()
            }
        }

        impl$(<$a>)? $name$(<$a>)? {
            #[doc = concat!("Creates an empty [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
            }

            #[doc = concat!("Initializes [`", stringify!($name), "`] with a specific element.")]
            #[must_use]
            #[inline]
            pub fn init(key: $key, value: $value) -> Self {
                Self::new().insert(key, value)
            }

            #[doc = concat!("Inserts a new element into [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, key: $key, value: $value) -> Self {
                let _ = self.0.insert(key, value);
                self
            }

            #[doc = concat!("Adds a new element into [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = concat!("Unlike [`Self::insert`], this method does not return a modified [`", stringify!($name), "`].")]
            #[inline]
            pub fn add(&mut self, key: $key, value: $value) {
                let _ = self.0.insert(key, value);
            }

            #[doc = concat!("Checks if [`", stringify!($name), "`] is empty.")]
            #[must_use]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Provides the number of elements in [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Returns an iterator over [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "**Iterates over the elements in the order they were inserted.**"]
            #[must_use]
            #[inline]
            pub fn iter(&self) -> Iter<'_, $key, $value> {
                self.0.iter()
            }
        }
    };
}

pub(crate) use map;
