use fenster_core::prelude::ParseError;
use kuchiki::{
    iter::{Descendants, Elements, Select},
    ElementData, NodeDataRef, NodeRef,
};

pub trait GetText {
    type Output;
    fn get_text(&self) -> Self::Output;
}

impl GetText for NodeDataRef<ElementData> {
    type Output = String;

    #[inline]
    fn get_text(&self) -> Self::Output {
        self.text_contents().clean_text()
    }
}

impl<T> GetText for Result<T, ()>
where
    T: GetText,
{
    type Output = Result<T::Output, ParseError>;

    fn get_text(&self) -> Self::Output {
        self.as_ref()
            .map(T::get_text)
            .map_err(|_| ParseError::ElementNotFound)
    }
}

pub trait OuterHtml {
    fn outer_html(&self) -> Result<String, ParseError>;
}

impl OuterHtml for NodeRef {
    fn outer_html(&self) -> Result<String, ParseError> {
        let mut out = Vec::new();
        self.serialize(&mut out)
            .map_err(|_| ParseError::SerializeFailed)?;
        Ok(String::from_utf8_lossy(&out).to_string())
    }
}

pub trait CollectText {
    fn collect_text(self) -> Vec<String>;
}

impl CollectText for Select<Elements<Descendants>> {
    fn collect_text(self) -> Vec<String> {
        self.map(|node| node.text_contents().clean_text())
            .collect::<Vec<_>>()
    }
}

impl<T> CollectText for Result<T, ()>
where
    T: CollectText,
{
    #[inline]
    fn collect_text(self) -> Vec<String> {
        self.map(T::collect_text).unwrap_or_default()
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

pub trait DetachAll {
    fn detach_all(self);
}

impl DetachAll for Select<Elements<Descendants>> {
    fn detach_all(self) {
        for node in self.collect::<Vec<_>>() {
            node.as_node().detach()
        }
    }
}

impl<T> DetachAll for Result<T, ()>
where
    T: DetachAll,
{
    fn detach_all(self) {
        if let Some(nodes) = self.ok() {
            nodes.detach_all();
        }
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

pub trait CleanText {
    fn clean_text(&self) -> String;
}

impl<T> CleanText for T
where
    T: AsRef<str>,
{
    #[inline]
    fn clean_text(&self) -> String {
        fn inner(value: &str) -> String {
            value
                .trim()
                .chars()
                .map(|c| match c {
                    ' ' | '\t' => ' ',
                    _ => c,
                })
                .collect()
        }
        inner(self.as_ref())
    }
}
