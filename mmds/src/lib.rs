// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;

pub mod data_store;

use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};

use data_store::{Error as MmdsError, Mmds};

lazy_static! {
    // A static reference to a global Mmds instance. We currently use this for ease of access during
    // prototyping. We'll consider something like passing Arc<Mutex<Mmds>> references to the
    // appropriate threads in the future.
    pub static ref MMDS: Arc<Mutex<Mmds>> = Arc::new(Mutex::new(Mmds::default()));
}

/// Patch provided JSON document (given as `serde_json::Value`) in-place with JSON Merge Patch
/// [RFC 7396](https://tools.ietf.org/html/rfc7396).
pub fn json_patch(target: &mut Value, patch: &Value) {
    if patch.is_object() {
        if !target.is_object() {
            // Replace target with a serde_json object so we can recursively copy patch values.
            *target = Value::Object(Map::new());
        }

        // This is safe since we make sure patch and target are objects beforehand.
        let doc = target.as_object_mut().unwrap();
        for (key, value) in patch.as_object().unwrap() {
            if value.is_null() {
                // If the value in the patch is null we remove the entry.
                doc.remove(key.as_str());
            } else {
                // Recursive call to update target document.
                // If `key` is not in the target document (it's a new field defined in `patch`)
                // insert a null placeholder and pass it as the new target
                // so we can insert new values recursively.
                json_patch(doc.entry(key.as_str()).or_insert(Value::Null), value);
            }
        }
    } else {
        *target = patch.clone();
    }
}

mod filters {
    use super::*;
    use warp::Filter;

    pub fn get_mds() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("mds")
            .and(warp::get())
            .and(warp::path::param())
            .and_then(handlers::get_mds_value)
    }

    pub fn put_mds() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("mds")
            .and(warp::path::end())
            .and(warp::put())
            .and(json_body())
            .and_then(handlers::put_mds)
    }

    fn json_body() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(150).and(warp::body::json())
    }
}

mod handlers {
    use super::*;
    use std::convert::Infallible;
    use warp::http::{Response, StatusCode};

    pub async fn get_mds_value(path: String) -> Result<impl warp::Reply, Infallible> {
        let value = MMDS
            .lock()
            .expect("Failed to build MMDS response due to poisoned lock")
            .get_value(path);
        let response = match value {
            Ok(v) => Response::builder()
                .status(StatusCode::OK)
                .body(serde_json::to_string(&v).unwrap()),

            Err(e) => match e {
                MmdsError::NotFound => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(format!("{}", MmdsError::NotFound)),
                MmdsError::UnsupportedValueType => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("{}", MmdsError::UnsupportedValueType)),
            },
        };

        Ok(response)
    }

    pub async fn put_mds() -> Result<impl warp::Reply, Infallible> {
        
        Ok("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_parse_request() {
        let data = r#"{
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": "43",
            "phones": {
                "home": {
                    "RO": "+40 1234567",
                    "UK": "+44 1234567"
                },
                "mobile": "+44 2345678"
            }
        }"#;
        MMDS.lock()
            .unwrap()
            .put_data(serde_json::from_str(data).unwrap())
            .unwrap();

        let data = r#"{
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": 43
        }"#;
        assert_eq!(
            MMDS.lock()
                .unwrap()
                .put_data(serde_json::from_str(data).unwrap()),
            Err(MmdsError::UnsupportedValueType)
        );
    }

    #[test]
    fn test_json_patch() {
        let mut data = serde_json::json!({
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": "43",
            "phones": {
                "home": {
                    "RO": "+40 1234567",
                    "UK": "+44 1234567"
                },
                "mobile": "+44 2345678"
            }
        });

        let patch = serde_json::json!({
            "name": {
                "second": null,
                "last": "Kennedy"
            },
            "age": "44",
            "phones": {
                "home": "+44 1234567",
                "mobile": {
                    "RO": "+40 2345678",
                    "UK": "+44 2345678"
                }
            }
        });
        json_patch(&mut data, &patch);

        // Test value replacement in target document.
        assert_eq!(data["age"], patch["age"]);

        // Test null value removal from target document.
        assert_eq!(data["name"]["second"], Value::Null);

        // Test add value to target document.
        assert_eq!(data["name"]["last"], patch["name"]["last"]);
        assert!(!data["phones"]["home"].is_object());
        assert_eq!(data["phones"]["home"], patch["phones"]["home"]);
        assert!(data["phones"]["mobile"].is_object());
        assert_eq!(
            data["phones"]["mobile"]["RO"],
            patch["phones"]["mobile"]["RO"]
        );
        assert_eq!(
            data["phones"]["mobile"]["UK"],
            patch["phones"]["mobile"]["UK"]
        );
    }
}
