use kuchiki::{ElementData, NodeDataRef, NodeRef};

pub trait SelectText {
    fn select_first_text(&self, selectors: &str) -> String;
    fn select_text(&self, selectors: &str) -> Vec<String>;
}

impl SelectText for NodeRef {
    fn select_first_text(&self, selectors: &str) -> String {
        self.select_first(selectors)
            .map(|node| node.text_contents().trim().to_string())
            .unwrap_or_default()
    }

    fn select_text(&self, selectors: &str) -> Vec<String> {
        self.select(selectors)
            .map(|nodes| {
                nodes
                    .map(|node| node.text_contents().trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}

pub trait GetAttribute {
    fn get_attribute(&self, key: &str) -> Option<String>;
}

impl GetAttribute for NodeDataRef<ElementData> {
    fn get_attribute(&self, key: &str) -> Option<String> {
        self.attributes
            .borrow()
            .get(key)
            .map(|value| value.to_string())
    }
}

impl<T> GetAttribute for Option<T>
where
    T: GetAttribute,
{
    fn get_attribute(&self, key: &str) -> Option<String> {
        self.as_ref()
            .map(|inner| inner.get_attribute(key))
            .flatten()
    }
}

impl<T> GetAttribute for Result<T, ()>
where
    T: GetAttribute,
{
    fn get_attribute(&self, key: &str) -> Option<String> {
        self.as_ref()
            .map(|inner| inner.get_attribute(key))
            .ok()
            .flatten()
    }
}

pub trait Transpose {
    type Output;
    fn transpose(self) -> Self::Output;
}

impl<T, E> Transpose for Option<Result<T, E>> {
    type Output = Result<Option<T>, E>;

    #[inline]
    fn transpose(self) -> Self::Output {
        self.map_or(Ok(None), |r| r.map(Some))
    }
}
