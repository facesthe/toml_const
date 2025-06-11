//! Normalizing module, aka field inference.
//!
//! Normalization is the process of inferring missing fields in a TOML table inside arrays.
//! A user can define only the fields they care about, and the rest are initialized with default values.
//!
//! Steps to perform normalization are:
//! - Derive a normalized "schema" from the input TOML table. Arrays are reduced to 0/1 elements
//! - Using this normalized table, visit the original table and populate missing fields
//!
//! Empty fields are populated with default values:
//! - primitive types are set to their defaults
//! - arrays are empty
//! - dates are set to `1970-01-01T00:00:00Z`

use std::collections::HashMap;

use proc_macro2 as pm2;
use proc_macro2::Span;
use quote::quote;
use toml::value::{Date, Datetime};

use crate::custom_struct::ConstIdentDef;

const DEFAULT_DATE: Date = Date {
    year: 1970,
    month: 1,
    day: 1,
};
const DEFAULT_TIME: toml::value::Time = toml::value::Time {
    hour: 0,
    minute: 0,
    second: 0,
    nanosecond: 0,
};
const DEFAULT_OFFSET: toml::value::Offset = toml::value::Offset::Z;

#[derive(Clone, Debug)]
pub enum NormalizationError {
    /// A mismatch in value types.
    ///
    /// Sequence of keys in reverse order that leads to this mismatch.
    ValueMismatch {
        /// Reverse key path leading to the mismatch
        path: Vec<String>,

        /// Conflicting value types
        value_types: (TomlValue, TomlValue),
    },
}

/// Working intermediate representation - contains only key and type information
#[derive(Clone, Debug, PartialEq)]
pub enum TomlValue {
    String,
    Integer,
    Float,
    Boolean,
    Datetime {
        date: bool,
        time: bool,
        offset: bool,
    },
    Array(Vec<TomlValue>),
    Table(HashMap<String, TomlValue>),
}

impl std::error::Error for NormalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalizationError::ValueMismatch { path, value_types } => {
                let path = path
                    .iter()
                    .rev()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("::");

                write!(
                    f,
                    "Value mismatch at {} - found: {:?} and {:?}",
                    path, value_types.0, value_types.1
                )
            }
        }
    }
}

impl From<TomlValue> for toml::Value {
    fn from(value: TomlValue) -> Self {
        match value {
            TomlValue::String => toml::Value::String(Default::default()),
            TomlValue::Integer => toml::Value::Integer(Default::default()),
            TomlValue::Float => toml::Value::Float(Default::default()),
            TomlValue::Boolean => toml::Value::Boolean(Default::default()),
            TomlValue::Datetime { date, time, offset } => {
                toml::Value::Datetime(toml::value::Datetime {
                    date: if date { Some(DEFAULT_DATE) } else { None },
                    time: if time { Some(DEFAULT_TIME) } else { None },
                    offset: if offset { Some(DEFAULT_OFFSET) } else { None },
                })
            }
            TomlValue::Array(elements) => {
                toml::Value::Array(elements.into_iter().map(|e| e.into()).collect())
            }
            TomlValue::Table(sub_table) => {
                toml::Value::Table(sub_table.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl From<toml::Value> for TomlValue {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(_) => Self::String,
            toml::Value::Integer(_) => Self::Integer,
            toml::Value::Float(_) => Self::Float,
            toml::Value::Boolean(_) => Self::Boolean,
            toml::Value::Datetime(datetime) => Self::Datetime {
                date: datetime.date.is_some(),
                time: datetime.time.is_some(),
                offset: datetime.offset.is_some(),
            },
            toml::Value::Array(values) => {
                Self::Array(values.into_iter().map(|v| v.into()).collect())
            }
            toml::Value::Table(map) => map.into(),
        }
    }
}

impl From<toml::Table> for TomlValue {
    fn from(value: toml::Table) -> Self {
        Self::Table(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl TomlValue {
    /// This method assumes that [TomlValue::normalize] is already called.
    pub fn normalize_toml(&self, toml: &mut toml::Value) {
        match (self, toml) {
            (TomlValue::String, toml::Value::String(_))
            | (TomlValue::Integer, toml::Value::Integer(_))
            | (TomlValue::Float, toml::Value::Float(_))
            | (TomlValue::Boolean, toml::Value::Boolean(_)) => (),

            (
                TomlValue::Datetime {
                    date: tv_date,
                    time: tv_time,
                    offset: tv_offset,
                },
                toml::Value::Datetime(Datetime { date, time, offset }),
            ) => {
                if *tv_date && date.is_none() {
                    *date = Some(DEFAULT_DATE)
                }

                if *tv_time && time.is_none() {
                    *time = Some(DEFAULT_TIME)
                }

                if *tv_offset && offset.is_none() {
                    *offset = Some(DEFAULT_OFFSET)
                }
            }
            (TomlValue::Array(toml_values), toml::Value::Array(values)) => {
                match toml_values.first() {
                    Some(toml_value) => {
                        for val in values {
                            toml_value.normalize_toml(val);
                        }
                    }
                    None => (),
                }
            }
            (TomlValue::Table(hash_map), toml::Value::Table(map)) => {
                for (key, value) in hash_map {
                    match (map.get_mut(key), value) {
                        (Some(toml_value), _) => {
                            value.normalize_toml(toml_value);
                        }
                        // for missing keys that point to arrays, we initialize them as empty arrays
                        (None, TomlValue::Array(_)) => {
                            map.insert(key.to_owned(), toml::Value::Array(vec![]));
                        }
                        (None, _) => {
                            map.insert(key.to_owned(), value.clone().into());
                        }
                    }
                }
            }

            _ => unimplemented!("normalizing different types cannot be done"),
        }
    }

    /// Derive a normalized version of [Self].
    ///
    /// At this point, the schema of [Self] will be superset of the original.
    pub fn normalize(&self) -> Result<Self, NormalizationError> {
        match self {
            TomlValue::Array(toml_values) => match toml_values.first() {
                Some(first) => {
                    let normalized = toml_values.iter().try_fold(first.clone(), |acc, item| {
                        let inter = item.normalize()?;
                        acc.union(&inter)
                    })?;

                    Ok(TomlValue::Array(vec![normalized]))
                }
                None => Ok(TomlValue::Array(vec![])),
            },

            TomlValue::Table(toml_table) => {
                let norm_table = toml_table
                    .iter()
                    .map(|(k, v)| {
                        let normalized_value = v.normalize();
                        match normalized_value {
                            Ok(nv) => Ok((k.clone(), nv)),
                            Err(e) => Err(e.propagate(k)),
                        }
                    })
                    .collect::<Result<HashMap<String, TomlValue>, NormalizationError>>()?;

                Ok(TomlValue::Table(norm_table))
            }

            TomlValue::Datetime { date, time, offset } => {
                Ok(Self::resolve_date_time_offset(*date, *time, *offset))
            }

            // everything else is already normalized
            other => Ok(other.clone()),
        }
    }

    /// Calculate the union of two [TomlValue] types.
    ///
    /// This will first check if both types are the same, and then merge table and array types.
    /// Arrays will be reduced to lengths 1 or 0.
    fn union(&self, other: &Self) -> Result<Self, NormalizationError> {
        match (self, other) {
            (TomlValue::String, TomlValue::String) => Ok(TomlValue::String),
            (TomlValue::Integer, TomlValue::Integer) => Ok(TomlValue::Integer),
            (TomlValue::Float, TomlValue::Float) => Ok(TomlValue::Float),
            (TomlValue::Boolean, TomlValue::Boolean) => Ok(TomlValue::Boolean),
            (
                TomlValue::Datetime {
                    date: ld,
                    time: lt,
                    offset: lo,
                },
                TomlValue::Datetime {
                    date: rd,
                    time: rt,
                    offset: ro,
                },
            ) => Ok(TomlValue::Datetime {
                date: *ld || *rd,
                time: *lt || *rt,
                offset: *lo || *ro,
            }),

            (TomlValue::Array(arr_self), TomlValue::Array(arr_other)) => {
                let mut chained = arr_self.iter().chain(arr_other.iter());

                match chained.next() {
                    Some(first) => {
                        let merged = arr_self
                            .iter()
                            .chain(arr_other.iter())
                            .try_fold(first.to_owned(), |acc, item| acc.union(item))?;

                        Ok(TomlValue::Array(vec![merged]))
                    }
                    None => Ok(TomlValue::Array(vec![])),
                }
            }

            (TomlValue::Table(tab_self), TomlValue::Table(tab_other)) => {
                let mut merged = tab_self.clone();

                for (key, value) in tab_other {
                    match merged.get_mut(key) {
                        Some(existing_val) => {
                            match existing_val.union(value) {
                                Ok(u) => *existing_val = u,
                                Err(e) => Err(e.propagate(key))?,
                            };
                        }
                        None => {
                            merged.insert(key.to_string(), value.clone());
                        }
                    }
                }

                Ok(TomlValue::Table(merged))
            }

            err_other => {
                println!("value types are not the same: {:?}", err_other);
                Err(NormalizationError::ValueMismatch {
                    path: vec![],
                    value_types: (err_other.0.clone(), err_other.1.clone()),
                })
            }
        }
    }

    /// Some date-time combinations are not valid
    fn resolve_date_time_offset(date: bool, time: bool, offset: bool) -> TomlValue {
        match (date, time, offset) {
            // offset date time - anything containing offsets is promoted to offset date time
            (_, _, true) => TomlValue::Datetime {
                date: true,
                time: true,
                offset: true,
            },
            // local date time
            (true, true, false) => TomlValue::Datetime {
                date: true,
                time: true,
                offset: false,
            },
            // local date
            (true, false, false) => TomlValue::Datetime {
                date: true,
                time: false,
                offset: false,
            },
            // local time
            (false, true, false) => TomlValue::Datetime {
                date: false,
                time: true,
                offset: false,
            },
            (false, false, false) => {
                unimplemented!("datetime cannot be constructed without any components")
            }
        }
    }

    /// Recursively define array and table types.
    ///
    /// `Self` should be normalized first.
    #[allow(unused)]
    fn definition(&self, key: &str) -> pm2::TokenStream {
        match self {
            TomlValue::String => quote! {&'static str},
            TomlValue::Integer => quote! {i64},
            TomlValue::Float => quote! {f64},
            TomlValue::Boolean => quote! {bool},
            TomlValue::Datetime { date, time, offset } => {
                let dt_ident = date_time_struct_ident(*date, *time, *offset);
                quote! {#dt_ident}
            }
            Self::Array(arr) => match arr.len() {
                0 => todo!("handle case - empty array inferred to be bool arr"),
                1 => {
                    let inner_value = &arr[0];

                    inner_value.definition(key)
                }
                _ => unimplemented!("normalized array should have 0 or 1 elements"),
            },
            Self::Table(tab) => {
                let self_ident = key.to_type_ident();
                let self_mod = key.to_module_ident();

                let mut x = 0;

                let fields = tab
                    .iter()
                    .map(|(k, v)| {
                        x += 1;
                        // let field_ident = k.to_variable_ident();
                        let field_ident = syn::Ident::new(&k, Span::call_site());

                        let field_type = match v {
                            TomlValue::String => quote! {&'static str},
                            TomlValue::Integer => quote! {i64},
                            TomlValue::Float => quote! {f64},
                            TomlValue::Boolean => quote! {bool},
                            TomlValue::Datetime { date, time, offset } => {
                                let dt_ident = date_time_struct_ident(*date, *time, *offset);
                                quote! {#dt_ident}
                            }
                            TomlValue::Array(_) => {
                                let id = k.to_type_ident();
                                quote! {&'static [#self_mod :: #id]}
                            }
                            TomlValue::Table(_) => {
                                let id = k.to_type_ident();
                                quote! {#self_mod :: #id}
                            }
                        };

                        quote! {
                            pub #field_ident: #field_type,
                        }
                    })
                    .collect::<pm2::TokenStream>();

                let inner_definitions = tab
                    .iter()
                    .filter(|(_, v)| matches!(v, TomlValue::Array(_) | TomlValue::Table(_)))
                    .map(|(k, v)| v.definition(k))
                    .collect::<pm2::TokenStream>();

                quote! {
                    // table definition
                    pub struct #self_ident {
                        #fields
                    }

                    pub mod #self_mod {
                        #inner_definitions
                    }
                }
            }
        }
    }
}

fn date_time_struct_ident(date: bool, time: bool, offset: bool) -> syn::Ident {
    match (date, time, offset) {
        (_, _, true) => syn::Ident::new("OffsetDateTime", Span::call_site()),
        (true, true, false) => syn::Ident::new("LocalDateTime", Span::call_site()),
        (true, false, false) => syn::Ident::new("LocalDate", Span::call_site()),
        (false, true, false) => syn::Ident::new("LocalTime", Span::call_site()),
        (false, false, false) => {
            unimplemented!("datetime cannot be constructed without any components")
        }
    }
}

impl NormalizationError {
    /// When receiving an error when performing some op on key+values, this function accumulates current key to the error.
    pub fn propagate(self, key: &str) -> NormalizationError {
        match self {
            // NormalizationError::KeyMismatch {
            //     path: mut tp,
            //     a_diff,
            //     b_diff,
            // } => {
            //     tp.push(key.to_string());

            //     NormalizationError::KeyMismatch {
            //         path: tp,
            //         a_diff,
            //         b_diff,
            //     }
            // }
            NormalizationError::ValueMismatch {
                mut path,
                value_types,
            } => {
                path.push(key.to_string());
                NormalizationError::ValueMismatch { path, value_types }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_toml_value_default() {
        let default_int = toml::Value::Integer(Default::default());
        println!("{}", default_int);

        // let default_dt = toml::value::Date::default();
    }

    #[test]
    fn test_parse_toml_toml_value() {
        const NORMALIZE_FILE: &str = include_str!("../../normalize.toml");

        let parsed = toml::Table::from_str(NORMALIZE_FILE).expect("must parse");
        let toml_val = TomlValue::from(parsed.clone());

        println!("original: {:#?}", toml_val);

        let normalized = match toml_val.normalize() {
            Ok(n) => n,
            Err(e) => panic!("{}", e),
        };
        println!("normalized: {:#?}", normalized);

        let mut og_value = toml::Value::Table(parsed.clone());
        normalized.normalize_toml(&mut og_value);
        let norm_table = og_value.as_table().unwrap();

        println!("norm table: {:#?}", norm_table);

        println!("definition: {}", normalized.definition("TOP_LEVEL_TABLE"));
    }
}
