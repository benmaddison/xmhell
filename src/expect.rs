use core::fmt;
use std::borrow::Cow;

use quick_xml::{events::Event, reader::Reader};

use crate::Error;

mod sealed {
    pub trait Sealed {}
    impl Sealed for quick_xml::Reader<&[u8]> {}
}
use self::sealed::Sealed;

/// Methods for traversing an XML documented in structured fashion.
///
/// # Examples
///
/// ``` rust
/// # use xmhell::{Error, Expect, quick_xml::{events::Event, Reader}};
/// let input = r#"
///     <root>
///         <ball>red</ball>
///         <bat/>
///         <ball>blue</ball>
///         <ball>green</ball>
///     </root>
/// "#;
///
/// let mut reader = Reader::from_str(input);
/// _ = reader.trim_text(true);
///
/// let mut balls = Vec::new();
///
/// reader.expect_element("root")?.read_inner(|reader| loop {
///     match reader.expect_element("ball") {
///         Ok(inner) => {
///             inner.read_inner(|reader| {
///                 balls.push(reader.expect_text()?.into_owned());
///                 Ok(())
///             })?;
///         }
///         Err(Error::Eof) => break Ok(()),
///         Err(Error::UnexpectedEvent(_)) => continue,
///         Err(err) => break Err(err.into()),
///     }
/// })?;
/// reader.expect_eof()?;
///
/// assert_eq!(balls, vec!["red", "blue", "green"]);
/// # Ok::<(), Error>(())
/// ```
pub trait Expect<'a>: Sealed {
    /// Attempt to match and consume a span `<name>...</name>`.
    ///
    /// On success an [`ElementReader`] is returned that can be used to read the
    /// child nodes of the matched element.
    ///
    /// # Errors
    ///
    /// An [`Error::Eof`] is returned if `self` reaches the end of it's input.
    /// This is useful to signal a containing loop to `break`.
    /// Otherwise, an [`Error::UnexpectedEvent`] is returned if the next
    /// [`Event`] encountered is not a start-tag for `name`.
    ///
    /// An [`Error::Reader`] is returned if an error is encountered while trying
    /// to read from `self`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// # use xmhell::{Error, Expect, quick_xml::{events::{BytesStart, Event}, Reader}};
    /// assert_eq!(
    ///     Reader::from_str("<node>contents</node>")
    ///         .expect_element("node")?
    ///         .read_inner(|reader| Ok(reader.expect_text()?.into_owned()))?,
    ///     "contents",
    /// );
    ///
    /// assert!(matches!(
    ///     Reader::from_str("<empty/>").expect_element("node"),
    ///     Err(Error::UnexpectedEvent(Event::Empty(_))),
    /// ));
    /// # Ok::<(), Error>(())
    /// ```
    fn expect_element(&mut self, name: &str) -> Result<ElementReader<'a, '_>, Error>;

    /// Attempt to match and consume an empty element `<name/>`.
    ///
    /// # Errors
    ///
    /// An [`Error::Eof`] is returned if `self` reaches the end of it's input.
    /// This is useful to signal a containing loop to `break`.
    /// Otherwise, an [`Error::UnexpectedEvent`] is returned if the next
    /// [`Event`] encountered is not an empty-tag for `name`.
    ///
    /// An [`Error::Reader`] is returned if an error is encountered while trying
    /// to read from `self`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// # use xmhell::{Error, Expect, quick_xml::{events::{BytesStart, Event}, Reader}};
    /// assert!(Reader::from_str("<empty/>").expect_empty("empty").is_ok());
    ///
    /// assert!(matches!(
    ///     Reader::from_str("<non-empty></non-empty>").expect_empty("empty"),
    ///     Err(Error::UnexpectedEvent(Event::Start(_))),
    /// ));
    /// # Ok::<(), Error>(())
    /// ```
    fn expect_empty(&mut self, name: &str) -> Result<(), Error>;

    /// Attempt to match and consume an [`Event::Eof`].
    ///
    /// # Errors
    ///
    /// An [`Error::UnexpectedEvent`] is returned if `self` is not at the end of
    /// its input.
    ///
    /// An [`Error::Reader`] is returned if an error is encountered while trying
    /// to read from `self`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// # use xmhell::{Error, Expect, quick_xml::{events::{BytesStart, Event}, Reader}};
    /// assert!(Reader::from_str("").expect_eof().is_ok());
    /// # Ok::<(), Error>(())
    /// ```
    fn expect_eof(&mut self) -> Result<(), Error>;

    /// Attempt to match and consume a text node.
    ///
    /// On success a [`Cow<'a, str>`][Cow] is returned with the un-escaped text
    /// of the node.
    ///
    /// # Errors
    ///
    /// An [`Error::Eof`] is returned if `self` reaches the end of it's input.
    /// This is useful to signal a containing loop to `break`.
    /// Otherwise, an [`Error::UnexpectedEvent`] is returned if the next
    /// [`Event`] encountered is not a text node.
    ///
    /// An [`Error::Reader`] is returned if an error is encountered while trying
    /// to read from `self`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// # use xmhell::{Error, Expect, quick_xml::{events::{BytesStart, Event}, Reader}};
    /// assert_eq!(
    ///     Reader::from_str("<root>This is &gt; than that</root>")
    ///         .expect_element("root")?
    ///         .read_inner(|reader| Ok(reader.expect_text()?.into_owned()))?,
    ///     "This is > than that"
    /// );
    /// # Ok::<(), Error>(())
    /// ```
    fn expect_text(&mut self) -> Result<Cow<'a, str>, Error>;
}

impl<'a> Expect<'a> for Reader<&'a [u8]> {
    fn expect_element(&mut self, name: &str) -> Result<ElementReader<'a, '_>, Error> {
        log::debug!("expecting element <{name}>");
        match self.read_event()? {
            Event::Start(tag) if tag.name().as_ref() == name.as_bytes() => {
                log::debug!("found element <{name}>, scanning for end tag");
                let end = tag.to_end();
                let span = self.read_text(end.name())?;
                log::debug!("found matching end tag, decoding contents");
                log::trace!("got contents {span}");
                Ok(ElementReader {
                    _parent: self,
                    span,
                })
            }
            Event::Eof => Err(Error::Eof),
            event => Err(Error::unexpected_event(event)),
        }
    }

    fn expect_empty(&mut self, name: &str) -> Result<(), Error> {
        log::debug!("expecting element <{name}/>");
        match self.read_event()? {
            Event::Empty(tag) if tag.name().as_ref() == name.as_bytes() => Ok(()),
            Event::Eof => Err(Error::Eof),
            event => Err(Error::unexpected_event(event)),
        }
    }

    fn expect_eof(&mut self) -> Result<(), Error> {
        log::debug!("expecting end-of-file");
        match self.read_event()? {
            Event::Eof => Ok(()),
            event => Err(Error::unexpected_event(event)),
        }
    }

    fn expect_text(&mut self) -> Result<Cow<'a, str>, Error> {
        log::debug!("expecting text node");
        match self.read_event()? {
            Event::Text(txt) => Ok(txt.unescape()?),
            Event::Eof => Err(Error::Eof),
            event => Err(Error::unexpected_event(event)),
        }
    }
}

/// An object providing access to the inner content of a non-leaf XML node,
/// returned by [`Expect::expect_element()`].
pub struct ElementReader<'a, 'b> {
    _parent: &'b mut Reader<&'a [u8]>,
    span: Cow<'a, str>,
}

impl ElementReader<'_, '_> {
    /// Consume the contents of `self` using a [`quick_xml::Reader`].
    ///
    /// See [`Expect`] for usage examples.
    ///
    /// # Errors
    ///
    /// An error is returned if the closure `f` returns an error, or if the
    /// [`quick_xml::Reader`] is not fully consumed by `f`.
    pub fn read_inner<F, T>(self, mut f: F) -> Result<T, Error>
    where
        F: FnMut(
            &mut Reader<&[u8]>,
        ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        let slice = self.span.as_ref();
        let mut reader = Reader::from_str(slice);
        _ = reader.trim_text(true);
        let result = f(&mut reader)?;
        reader.expect_eof()?;
        Ok(result)
    }
}

impl fmt::Debug for ElementReader<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementReader")
            .field("span", &self.span)
            .finish()
    }
}
