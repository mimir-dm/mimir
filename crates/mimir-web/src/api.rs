//! A thin client over the same-origin API of mimir-server. Each call sends
//! the stored DM token (if any) as a bearer token. Errors are Aurora's
//! `ApiError`, filled from the server's `mimir_wire::ErrorBody`, so pages
//! show them with `ErrorState`.

use aurora_leptos::tokens::ApiError;
use gloo_net::http::{Request, RequestBuilder};
use mimir_wire::{self as wire, ErrorBody, Session, WebConfig};
use serde::de::DeserializeOwned;

use crate::auth;

/// `GET /api/config`: the server version and whether a token is needed.
pub async fn config() -> Result<WebConfig, ApiError> {
    get_json(Request::get("/api/config")).await
}

/// `GET /api/v1/session` with the stored token.
pub async fn session() -> Result<Session, ApiError> {
    get_json(with_token(
        Request::get("/api/v1/session"),
        auth::stored_token(),
    ))
    .await
}

/// `GET /api/v1/session` with a token that is not stored yet (sign-in).
pub async fn check_token(token: &str) -> Result<Session, ApiError> {
    get_json(with_token(
        Request::get("/api/v1/session"),
        Some(token.to_string()),
    ))
    .await
}

/// `GET` a path under `/api/v1` with the stored token.
pub async fn get<T: DeserializeOwned>(path: impl AsRef<str>) -> Result<T, ApiError> {
    let url = format!("/api/v1{}", path.as_ref());
    get_json(with_token(Request::get(&url), auth::stored_token())).await
}

/// The campaigns (`archived`: with the archived ones).
pub async fn campaigns(archived: bool) -> Result<Vec<wire::CampaignSummary>, ApiError> {
    get(if archived {
        "/campaigns?archived=true"
    } else {
        "/campaigns"
    })
    .await
}

pub async fn campaign(id: String) -> Result<wire::CampaignSummary, ApiError> {
    get(&format!("/campaigns/{id}")).await
}

/// A list under a campaign: "documents", "modules", "pcs", "npcs", "maps".
pub async fn campaign_list<T: DeserializeOwned>(
    id: String,
    what: &str,
) -> Result<Vec<T>, ApiError> {
    get(&format!("/campaigns/{id}/{what}")).await
}

pub async fn module(id: String) -> Result<wire::ModuleSummary, ApiError> {
    get(&format!("/modules/{id}")).await
}

/// A list under a module: "documents", "monsters", "npcs", "maps".
pub async fn module_list<T: DeserializeOwned>(id: String, what: &str) -> Result<Vec<T>, ApiError> {
    get(&format!("/modules/{id}/{what}")).await
}

pub async fn document(id: String) -> Result<wire::Document, ApiError> {
    get(&format!("/documents/{id}")).await
}

/// Send JSON with a method ("POST", "PATCH", "PUT") to a path under
/// `/api/v1`, and read the JSON answer.
pub async fn send<B: serde::Serialize, T: DeserializeOwned>(
    method: &str,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let req = match method {
        "POST" => Request::post(&url),
        "PUT" => Request::put(&url),
        "PATCH" => Request::patch(&url),
        _ => return Err(ApiError::Unknown(format!("method {method}"))),
    };
    let req = with_token(req, auth::stored_token())
        .json(body)
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    let resp = req.send().await.map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if resp.ok() {
        return resp
            .json::<T>()
            .await
            .map_err(|e| ApiError::Unknown(format!("unexpected response: {e}")));
    }
    let text = resp.text().await.unwrap_or_default();
    Err(decode_error(status, &text))
}

/// `POST` a form (a file upload) to a path under `/api/v1`. The browser
/// sets the multipart content type.
pub async fn upload<T: DeserializeOwned>(
    path: &str,
    form: &web_sys::FormData,
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let req = with_token(Request::post(&url), auth::stored_token())
        .body(wasm_bindgen::JsValue::from(form.clone()))
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    let resp = req.send().await.map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if resp.ok() {
        return resp
            .json::<T>()
            .await
            .map_err(|e| ApiError::Unknown(format!("unexpected response: {e}")));
    }
    let text = resp.text().await.unwrap_or_default();
    Err(decode_error(status, &text))
}

/// `POST` a form with upload progress (MIMIR-T-0724): `on_progress`
/// gets 0.0–1.0 as the bytes go up. XHR, as fetch reports no upload
/// progress.
pub async fn upload_with_progress<T: DeserializeOwned>(
    path: &str,
    form: &web_sys::FormData,
    on_progress: impl Fn(f64) + 'static,
) -> Result<T, ApiError> {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let url = format!("/api/v1{path}");
    let xhr = web_sys::XmlHttpRequest::new().map_err(|_| ApiError::Unknown("xhr".into()))?;
    xhr.open("POST", &url)
        .map_err(|_| ApiError::Unknown("xhr open".into()))?;
    if let Some(t) = auth::stored_token() {
        let _ = xhr.set_request_header("Authorization", &bearer(&t));
    }
    if let Ok(up) = xhr.upload() {
        let cb =
            Closure::<dyn Fn(web_sys::ProgressEvent)>::new(move |e: web_sys::ProgressEvent| {
                if e.length_computable() && e.total() > 0.0 {
                    on_progress(e.loaded() / e.total());
                }
            });
        up.set_onprogress(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
    }
    let done = {
        let xhr = xhr.clone();
        js_sys::Promise::new(&mut move |resolve, reject| {
            let ok = Closure::<dyn Fn()>::new(move || {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            });
            let fail = Closure::<dyn Fn()>::new(move || {
                let _ = reject.call0(&wasm_bindgen::JsValue::NULL);
            });
            xhr.set_onload(Some(ok.as_ref().unchecked_ref()));
            xhr.set_onerror(Some(fail.as_ref().unchecked_ref()));
            ok.forget();
            fail.forget();
        })
    };
    xhr.send_with_opt_form_data(Some(form))
        .map_err(|_| ApiError::Network)?;
    wasm_bindgen_futures::JsFuture::from(done)
        .await
        .map_err(|_| ApiError::Network)?;
    let status = xhr.status().unwrap_or(0);
    let text = xhr.response_text().ok().flatten().unwrap_or_default();
    if (200..300).contains(&status) {
        serde_json::from_str(&text)
            .map_err(|e| ApiError::Unknown(format!("unexpected response: {e}")))
    } else {
        Err(decode_error(status, &text))
    }
}

/// The file name in a `Content-Disposition` header.
pub fn attachment_name(header: &str) -> Option<String> {
    let i = header.find("filename=")?;
    let rest = header[i + 9..].trim();
    let name = rest.trim_start_matches('"');
    let end = name.find('"').unwrap_or(name.len());
    let name = name[..end].trim_end_matches(';').trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// `GET` a file under `/api/v1` (with the token) and save it as a
/// download (MIMIR-T-0724: archives, PDFs).
pub async fn download(path: &str, fallback_name: &str) -> Result<(), ApiError> {
    use wasm_bindgen::JsCast;

    let url = format!("/api/v1{path}");
    let resp = with_token(Request::get(&url), auth::stored_token())
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(decode_error(status, &text));
    }
    let name = resp
        .headers()
        .get("content-disposition")
        .and_then(|h| attachment_name(&h))
        .unwrap_or_else(|| fallback_name.to_string());
    let mime = resp
        .headers()
        .get("content-type")
        .unwrap_or_else(|| "application/octet-stream".into());
    let bytes = resp
        .binary()
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    let array = js_sys::Uint8Array::from(bytes.as_slice());
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(&mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &js_sys::Array::of1(&array),
        &options,
    )
    .map_err(|_| ApiError::Unknown("blob".into()))?;
    let href = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| ApiError::Unknown("object url".into()))?;
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| ApiError::Unknown("no document".into()))?;
    let a = doc
        .create_element("a")
        .map_err(|_| ApiError::Unknown("anchor".into()))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| ApiError::Unknown("anchor".into()))?;
    a.set_href(&href);
    a.set_download(&name);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&href);
    Ok(())
}

/// `DELETE` a path under `/api/v1`.
pub async fn delete(path: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1{path}");
    let resp = with_token(Request::delete(&url), auth::stored_token())
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    if resp.ok() {
        return Ok(());
    }
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    Err(decode_error(status, &text))
}

/// `DELETE` a path under `/api/v1` that answers JSON.
pub async fn delete_json<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    get_json(with_token(Request::delete(&url), auth::stored_token())).await
}

/// An image under `/api/v1` as an object URL (`blob:`), for `<image href>`.
/// Images need the token, which an `<img src>` cannot send. `Ok(None)`:
/// there is no image (404).
pub async fn image_url(path: impl AsRef<str>) -> Result<Option<String>, ApiError> {
    let url = format!("/api/v1{}", path.as_ref());
    let resp = with_token(Request::get(&url), auth::stored_token())
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    if resp.status() == 404 {
        return Ok(None);
    }
    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(decode_error(status, &text));
    }
    let mime = resp
        .headers()
        .get("content-type")
        .unwrap_or_else(|| "application/octet-stream".into());
    let bytes = resp
        .binary()
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    let array = js_sys::Uint8Array::from(bytes.as_slice());
    let parts = js_sys::Array::of1(&array);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(&mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|_| ApiError::Unknown("blob".into()))?;
    web_sys::Url::create_object_url_with_blob(&blob)
        .map(Some)
        .map_err(|_| ApiError::Unknown("object url".into()))
}

fn with_token(req: RequestBuilder, token: Option<String>) -> RequestBuilder {
    match token {
        Some(t) => req.header("Authorization", &bearer(&t)),
        None => req,
    }
}

/// The `Authorization` header value for a token.
pub fn bearer(token: &str) -> String {
    format!("Bearer {}", token.trim())
}

async fn get_json<T: DeserializeOwned>(req: RequestBuilder) -> Result<T, ApiError> {
    let resp = req.send().await.map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if resp.ok() {
        return resp
            .json::<T>()
            .await
            .map_err(|e| ApiError::Unknown(format!("unexpected response: {e}")));
    }
    let text = resp.text().await.unwrap_or_default();
    Err(decode_error(status, &text))
}

/// An error response as Aurora's `ApiError`: the code and the message of the
/// envelope when the body is one, else the status alone.
pub fn decode_error(status: u16, body: &str) -> ApiError {
    match serde_json::from_str::<ErrorBody>(body) {
        Ok(env) => ApiError::Http {
            status,
            message: env.error.message,
            code: serde_json::to_value(env.error.code)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string)),
        },
        Err(_) => ApiError::Http {
            status,
            message: if body.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                body.trim().to_string()
            },
            code: None,
        },
    }
}

/// The text of an error, for a person.
pub fn message(e: ApiError) -> String {
    match e {
        ApiError::Http { message, .. } => message,
        ApiError::Network => "The server did not answer.".into(),
        ApiError::Unknown(m) => m,
    }
}

/// Is this a refused token (401)?
pub fn is_unauthorized(err: &ApiError) -> bool {
    matches!(err, ApiError::Http { status: 401, .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_wire::ErrorCode;

    #[test]
    fn an_envelope_decodes_to_its_code_and_message() {
        let body =
            serde_json::to_string(&ErrorBody::new(ErrorCode::Validation, "name is empty")).unwrap();
        assert!(
            decode_error(422, &body)
                == ApiError::Http {
                    status: 422,
                    message: "name is empty".into(),
                    code: Some("validation".into()),
                }
        );
    }

    #[test]
    fn a_body_that_is_not_an_envelope_keeps_the_status() {
        assert!(
            decode_error(502, "")
                == ApiError::Http {
                    status: 502,
                    message: "HTTP 502".into(),
                    code: None
                }
        );
        assert!(
            decode_error(500, "boom\n")
                == ApiError::Http {
                    status: 500,
                    message: "boom".into(),
                    code: None
                }
        );
    }

    #[test]
    fn only_401_is_a_refused_token() {
        assert!(is_unauthorized(&decode_error(401, "")));
        assert!(!is_unauthorized(&decode_error(403, "")));
        assert!(!is_unauthorized(&ApiError::Network));
    }

    #[test]
    fn the_bearer_header_trims_the_token() {
        assert_eq!(bearer("  abc \n"), "Bearer abc");
    }
}
