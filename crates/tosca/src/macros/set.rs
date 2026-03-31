macro_rules! set {
    (
        /// $desc:expr
        $(#[$attrs:meta])*
        pub struct $name:ident(IndexSet<$element:ty, DefaultHashBuilder>);
    ) => {
        ///
        $(#[$attrs])*
        pub struct $name(IndexSet<$element, DefaultHashBuilder>);

        impl IntoIterator for $name {
            type Item = $element;
            type IntoIter = IntoIter<$element>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = &'a $element;
            type IntoIter = Iter<'a, $element>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            #[doc = concat!("Creates an empty [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(IndexSet::default())
            }

            #[doc = concat!("Initializes [`", stringify!($name), "`] with a specific element.")]
            #[must_use]
            #[inline]
            pub fn init(element: $element) -> Self {
                Self::new().insert(element)
            }

            #[doc = concat!("Inserts a new element into [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, element: $element) -> Self {
                let _ = self.0.insert(element);
                self
            }

            #[doc = concat!("Adds a new element into [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = concat!("Unlike [`Self::insert`], this method does not return a modified [`", stringify!($name), "`].")]
            #[inline]
            pub fn add(&mut self, element: $element) {
                let _ = self.0.insert(element);
            }

            #[doc = concat!("Checks if [`", stringify!($name), "`] contains the given [`", stringify!($element), "`].")]
            #[inline]
            #[must_use]
            pub fn contains(&self, hazard: &$element) -> bool {
                self.0.contains(hazard)
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

            #[doc = concat!("Gets an element from [`", stringify!($name), "`] by index.")]
            #[inline]
            pub fn get_index(&self, index: usize) -> Option<&$element> {
                self.0.get_index(index)
            }

            #[doc = concat!("Returns an iterator over [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "**Iterates over the elements in the order they were inserted.**"]
            #[must_use]
            #[inline]
            pub fn iter(&self) -> Iter<'_, $element> {
                self.0.iter()
            }
        }
    };
}

pub(crate) use set;
