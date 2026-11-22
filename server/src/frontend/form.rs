//! Form deserialization for the legacy HTMX UI.
//!
//! [`axum::Form`] uses `serde_urlencoded`, which is a poor fit for HTML
//! checkbox groups:
//!
//! * repeated keys (`roles=manager&roles=cook`) fail with
//!   `duplicate field`,
//! * a group with exactly one checked box arrives as a bare string and
//!   fails `Vec<T>` with `invalid type: string, expected a sequence`.
//!
//! [`serde_html_form`] implements HTML form semantics instead: repeated
//! keys deserialize into `Vec<T>` and a single value becomes a
//! one-element vec. The [`HtmlForm`] extractor below is a drop-in
//! replacement for `axum::Form` on handlers that receive checkbox groups
//! (`roles`, `permissions`, …); use it together with
//! `#[serde(default)]` so an entirely unchecked group deserializes to an
//! empty vec (meaning "no roles granted", not a client error).

use axum::extract::{FromRequest, RawForm, Request};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

/// Deserializes a form field into `Vec<T>`, accepting either a sequence of
/// values or a single value. Only needed for deserializers that do not
/// implement HTML form semantics (e.g. `serde_urlencoded`); with
/// [`HtmlForm`] a plain `#[serde(default)]` suffices.
pub fn seq_or_single<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct SeqOrSingle<T>(std::marker::PhantomData<T>);

    impl<'de, T> serde::de::Visitor<'de> for SeqOrSingle<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a sequence of values or a single value")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut out = Vec::new();
            while let Some(value) = seq.next_element::<T>()? {
                out.push(value);
            }
            Ok(out)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![T::deserialize(
                serde::de::value::StrDeserializer::<E>::new(value),
            )?])
        }
    }

    deserializer.deserialize_any(SeqOrSingle(std::marker::PhantomData))
}

/// `FromRequest` extractor that parses the request body with
/// [`serde_html_form`]. Drop-in replacement for `axum::Form` wherever a
/// form contains checkbox groups.
pub struct HtmlForm<T>(pub T);

impl<S, T> FromRequest<S> for HtmlForm<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let RawForm(bytes) = RawForm::from_request(req, state)
            .await
            .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;
        parse(&bytes)
            .map(HtmlForm)
            .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))
    }
}

fn parse<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, serde_html_form::de::Error> {
    serde_html_form::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Form {
        #[serde(default)]
        values: Vec<String>,
    }

    #[test]
    fn single_checked_box_becomes_one_element_vec() {
        let form: Form = parse(b"values=manager").unwrap();
        assert_eq!(form.values, vec!["manager".to_string()]);
    }

    #[test]
    fn repeated_keys_become_a_vec() {
        let form: Form = parse(b"values=manager&values=cook").unwrap();
        assert_eq!(form.values, vec!["manager".to_string(), "cook".to_string()]);
    }

    #[test]
    fn no_checked_box_deserializes_to_empty() {
        let form: Form = parse(b"other=x").unwrap();
        assert!(form.values.is_empty());
    }

    #[test]
    fn seq_or_single_also_accepts_both_shapes() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct UrlencodedForm {
            #[serde(default, deserialize_with = "super::seq_or_single")]
            values: Vec<String>,
        }
        let one: UrlencodedForm = serde_urlencoded::from_str("values=manager").unwrap();
        assert_eq!(one.values, vec!["manager".to_string()]);
    }
}

