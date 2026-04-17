use serde::{Deserialize, Deserializer, Serialize};
#[cfg(feature = "macro")]
pub use serde_tristate_macros::serde_tristate;

/// Three-state value for HTTP PATCH request bodies.
///
/// - `Value(T)` — field present with a value
/// - `None` — field present as JSON `null`
/// - `Undefined` — field absent from the payload
///
/// # Serde integration
///
/// Annotate the containing struct/enum with `#[serde_tristate]` and derive
/// `Serialize`/`Deserialize` normally — no per-field attributes needed.
///
/// ```ignore
/// #[serde_tristate]
/// #[derive(Serialize, Deserialize)]
/// struct UpdateUser {
///     name: Tristate<String>,
///     age:  Tristate<u32>,
/// }
/// ```
///
#[derive(Default)]
pub enum Tristate<T> {
    Value(T),
    None,
    #[default]
    Undefined,
}

impl<T> Tristate<T> {
    pub fn is_undefined(&self) -> bool {
        matches!(self, Tristate::Undefined)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Tristate::None)
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Tristate::Value(_))
    }
}

impl<T> From<T> for Tristate<T> {
    fn from(v: T) -> Self {
        Tristate::Value(v)
    }
}

impl<T> From<Option<T>> for Tristate<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Tristate::Value(v),
            None => Tristate::None,
        }
    }
}

impl<T> From<Option<Option<T>>> for Tristate<T> {
    fn from(opt: Option<Option<T>>) -> Self {
        match opt {
            None => Tristate::Undefined,
            Some(None) => Tristate::None,
            Some(Some(v)) => Tristate::Value(v),
        }
    }
}

impl<T> From<Tristate<T>> for Option<Option<T>> {
    fn from(val: Tristate<T>) -> Self {
        match val {
            Tristate::Undefined => None,
            Tristate::None => Some(None),
            Tristate::Value(v) => Some(Some(v)),
        }
    }
}

impl<T> Tristate<T> {
    /// Map `Value(v)` through `f`. `None` and `Undefined` pass through unchanged.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Tristate<U> {
        match self {
            Tristate::Value(v) => Tristate::Value(f(v)),
            Tristate::None => Tristate::None,
            Tristate::Undefined => Tristate::Undefined,
        }
    }

    /// Chain on `Value(v)`. `None` and `Undefined` pass through unchanged.
    pub fn and_then<U, F: FnOnce(T) -> Tristate<U>>(self, f: F) -> Tristate<U> {
        match self {
            Tristate::Value(v) => f(v),
            Tristate::None => Tristate::None,
            Tristate::Undefined => Tristate::Undefined,
        }
    }

    /// Return the contained value or `default` if `None` / `Undefined`.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Tristate::Value(v) => v,
            _ => default,
        }
    }

    /// Return the contained value or compute it from `f` if `None` / `Undefined`.
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Tristate::Value(v) => v,
            _ => f(),
        }
    }
}

impl<T> Tristate<T> {
    /// Apply to a required target field.
    /// `Value` overwrites; `None` and `Undefined` are no-ops.
    pub fn apply_to_tristate(self, target: &mut T) {
        if let Tristate::Value(v) = self {
            *target = v;
        }
    }

    /// Apply to an optional target field.
    /// `Value(v)` → `Some(v)`, `None` → `None`, `Undefined` → no-op.
    pub fn apply_to_option(self, target: &mut Option<T>) {
        match self {
            Tristate::Value(v) => *target = Some(v),
            Tristate::None => *target = None,
            Tristate::Undefined => {}
        }
    }
}

impl<T: Serialize> Serialize for Tristate<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Tristate::Value(v) => v.serialize(serializer),
            Tristate::None => serializer.serialize_none(),
            Tristate::Undefined => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Tristate<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|opt| match opt {
            Some(v) => Tristate::Value(v),
            None => Tristate::None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[serde_tristate]
    #[derive(Serialize, Deserialize)]
    struct Dto {
        name: Tristate<String>,
        age: Tristate<u32>,
    }

    #[test]
    fn serialize_value() {
        let dto = Dto {
            name: Tristate::Value("Alice".into()),
            age: Tristate::Undefined,
        };
        assert_eq!(serde_json::to_string(&dto).unwrap(), r#"{"name":"Alice"}"#);
    }

    #[test]
    fn serialize_none() {
        let dto = Dto {
            name: Tristate::None,
            age: Tristate::Undefined,
        };
        assert_eq!(serde_json::to_string(&dto).unwrap(), r#"{"name":null}"#);
    }

    #[test]
    fn serialize_undefined_skipped() {
        let dto = Dto {
            name: Tristate::Undefined,
            age: Tristate::Undefined,
        };
        assert_eq!(serde_json::to_string(&dto).unwrap(), r#"{}"#);
    }

    #[test]
    fn deserialize_value() {
        let dto: Dto = serde_json::from_str(r#"{"name":"Bob","age":42}"#).unwrap();
        assert!(matches!(dto.name, Tristate::Value(s) if s == "Bob"));
        assert!(matches!(dto.age, Tristate::Value(42)));
    }

    #[test]
    fn deserialize_null_as_none() {
        let dto: Dto = serde_json::from_str(r#"{"name":null}"#).unwrap();
        assert!(matches!(dto.name, Tristate::None));
        assert!(matches!(dto.age, Tristate::Undefined));
    }

    #[test]
    fn deserialize_absent_as_undefined() {
        let dto: Dto = serde_json::from_str(r#"{}"#).unwrap();
        assert!(matches!(dto.name, Tristate::Undefined));
        assert!(matches!(dto.age, Tristate::Undefined));
    }

    #[serde_tristate]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "kind")]
    enum Event {
        Update {
            name: Tristate<String>,
            age: Tristate<u32>,
        },
        Delete,
    }

    #[test]
    fn enum_serialize_value() {
        let e = Event::Update {
            name: Tristate::Value("Carol".into()),
            age: Tristate::Undefined,
        };
        assert_eq!(
            serde_json::to_string(&e).unwrap(),
            r#"{"kind":"Update","name":"Carol"}"#
        );
    }

    #[test]
    fn enum_serialize_all_undefined() {
        let e = Event::Update {
            name: Tristate::Undefined,
            age: Tristate::Undefined,
        };
        assert_eq!(serde_json::to_string(&e).unwrap(), r#"{"kind":"Update"}"#);
    }

    #[test]
    fn enum_deserialize_value() {
        let e: Event = serde_json::from_str(r#"{"kind":"Update","name":"Dave"}"#).unwrap();
        assert!(
            matches!(e, Event::Update { name: Tristate::Value(s), age: Tristate::Undefined } if s == "Dave")
        );
    }

    #[test]
    fn from_value() {
        let p: Tristate<i32> = 42.into();
        assert!(matches!(p, Tristate::Value(42)));
    }

    #[test]
    fn from_some() {
        let p: Tristate<i32> = Some(42).into();
        assert!(matches!(p, Tristate::Value(42)));
    }

    #[test]
    fn from_option_none() {
        let p: Tristate<i32> = Option::<i32>::None.into();
        assert!(matches!(p, Tristate::None));
    }

    #[test]
    fn into_option_value() {
        assert_eq!(
            Into::<Option<Option<i32>>>::into(Tristate::Value(1)),
            Some(Some(1))
        );
    }

    #[test]
    fn into_option_none() {
        assert_eq!(
            Into::<Option<Option<i32>>>::into(Tristate::<i32>::None),
            Some(Option::None)
        );
    }

    #[test]
    fn into_option_undefined() {
        assert_eq!(
            Into::<Option<Option<i32>>>::into(Tristate::<i32>::Undefined),
            Option::<Option<i32>>::None
        );
    }

    #[test]
    fn from_option_fn_some_some() {
        assert!(matches!(Tristate::from(Some(Some(1))), Tristate::Value(1)));
    }

    #[test]
    fn from_option_fn_some_none() {
        assert!(matches!(
            Tristate::<i32>::from(Some(Option::None)),
            Tristate::None
        ));
    }

    #[test]
    fn from_option_fn_outer_none() {
        assert!(matches!(
            Tristate::<i32>::from(Option::<Option<i32>>::None),
            Tristate::Undefined
        ));
    }

    #[test]
    fn from_option_roundtrip() {
        let cases: [Tristate<i32>; 3] = [Tristate::Value(42), Tristate::None, Tristate::Undefined];
        for t in cases {
            let opt: Option<Option<i32>> = t.into();
            assert!(matches!(Tristate::<i32>::from(opt), _));
        }
    }

    #[test]
    fn map_value() {
        assert!(matches!(
            Tristate::Value(2).map(|x| x * 3),
            Tristate::Value(6)
        ));
    }

    #[test]
    fn map_none_passthrough() {
        assert!(matches!(
            Tristate::<i32>::None.map(|x| x * 3),
            Tristate::None
        ));
    }

    #[test]
    fn and_then_value() {
        let p = Tristate::Value(5).and_then(|x| {
            if x > 3 {
                Tristate::Value(x)
            } else {
                Tristate::None
            }
        });
        assert!(matches!(p, Tristate::Value(5)));
    }

    #[test]
    fn unwrap_or_value() {
        assert_eq!(Tristate::Value(7).unwrap_or(0), 7);
    }

    #[test]
    fn unwrap_or_undefined() {
        assert_eq!(Tristate::<i32>::Undefined.unwrap_or(0), 0);
    }

    #[test]
    fn apply_to_required_sets_value() {
        let mut target = String::from("old");
        Tristate::Value("new".to_string()).apply_to_tristate(&mut target);
        assert_eq!(target, "new");
    }

    #[test]
    fn apply_to_required_undefined_noop() {
        let mut target = String::from("old");
        Tristate::<String>::Undefined.apply_to_tristate(&mut target);
        assert_eq!(target, "old");
    }

    #[test]
    fn apply_to_option_sets_some() {
        let mut target: Option<String> = Option::None;
        Tristate::Value("hi".to_string()).apply_to_option(&mut target);
        assert_eq!(target, Some("hi".to_string()));
    }

    #[test]
    fn apply_to_option_none_clears() {
        let mut target: Option<String> = Some("bye".to_string());
        Tristate::<String>::None.apply_to_option(&mut target);
        assert_eq!(target, Option::None);
    }

    #[test]
    fn apply_to_option_undefined_noop() {
        let mut target: Option<String> = Some("keep".to_string());
        Tristate::<String>::Undefined.apply_to_option(&mut target);
        assert_eq!(target, Some("keep".to_string()));
    }
}
