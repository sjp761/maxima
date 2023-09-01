use core::slice;
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_uint, c_void},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use maxima::{
    core::{
        auth::login,
        background_service::request_registry_setup,
        launch,
        service_layer::{send_service_request, GraphQLRequest, ServiceGetBasicPlayerRequest},
        Maxima, MaximaEvent,
    },
    util::{
        registry::check_registry_validity,
        service::{is_service_running, is_service_valid, register_service_user, start_service},
    },
};
use tokio::{runtime::Runtime, sync::Mutex};

pub const ERR_SUCCESS: usize = 0;
pub const ERR_UNKNOWN: usize = 1;
pub const ERR_LOGIN_FAILED: usize = 2;
pub const ERR_INVALID_ARGUMENT: usize = 3;

/// Create an asynchronous runtime.
#[no_mangle]
pub extern "C" fn maxima_create_runtime(runtime_out: *mut *mut c_void) -> usize {
    let result = Runtime::new();
    if result.is_err() {
        return ERR_UNKNOWN;
    }

    let runtime = Box::new(result.unwrap());
    unsafe { *runtime_out = Box::into_raw(runtime) as *mut c_void }

    ERR_SUCCESS
}

/// Check if the Maxima Background Service is installed and valid.
#[no_mangle]
pub extern "C" fn maxima_is_service_valid(out: *mut bool) -> usize {
    let result = is_service_valid();
    if result.is_err() {
        return ERR_UNKNOWN;
    }

    unsafe { *out = result.unwrap() };
    ERR_SUCCESS
}

/// Check if the Maxima Background Service is running.
#[no_mangle]
pub extern "C" fn maxima_is_service_running(out: *mut bool) -> usize {
    let result = is_service_running();
    if result.is_err() {
        return ERR_UNKNOWN;
    }

    unsafe { *out = result.unwrap() };
    ERR_SUCCESS
}

/// Register the Maxima Background Service. Runs maxima-bootstrap for admin access.
#[no_mangle]
pub extern "C" fn maxima_register_service() -> usize {
    let result = register_service_user();
    if result.is_err() {
        return ERR_UNKNOWN;
    }

    ERR_SUCCESS
}

/// Start the Maxima Background Service
#[no_mangle]
pub extern "C" fn maxima_start_service(runtime: *mut *mut Runtime) -> usize {
    let rt = unsafe { Box::from_raw(*runtime) };
    let result = rt.block_on(async { start_service().await });

    if result.is_err() {
        return ERR_UNKNOWN;
    }

    unsafe {
        *runtime = Box::into_raw(rt);
    }

    ERR_SUCCESS
}

/// Check if the Windows Registry is properly set up for Maxima
#[no_mangle]
pub extern "C" fn maxima_check_registry_validity() -> bool {
    check_registry_validity().is_ok()
}

/// Request the Maxima Background Service to set up the Windows Registry
#[no_mangle]
pub extern "C" fn maxima_request_registry_setup(runtime: *mut *mut Runtime) -> usize {
    let rt = unsafe { Box::from_raw(*runtime) };
    let result = rt.block_on(async { request_registry_setup().await });

    if result.is_err() {
        return ERR_UNKNOWN;
    }

    unsafe { *runtime = Box::into_raw(rt) }

    ERR_SUCCESS
}

/// Log into an EA account and retrieve an access token. Opens the EA website for authentication.
#[no_mangle]
pub extern "C" fn maxima_login(runtime: *mut *mut Runtime, token_out: *mut *mut c_char) -> usize {
    let rt = unsafe { Box::from_raw(*runtime) };

    let result = rt.block_on(async { login::execute().await });
    if result.is_err() {
        return ERR_UNKNOWN;
    }

    let token = result.unwrap();
    if token.is_none() {
        return ERR_LOGIN_FAILED;
    }

    let raw_token = CString::new(token.unwrap());
    if raw_token.is_err() {
        return ERR_UNKNOWN;
    }

    unsafe {
        *runtime = Box::into_raw(rt);
        *token_out = raw_token.unwrap().into_raw();
    }

    ERR_SUCCESS
}

/// Creates a Maxima object.
#[no_mangle]
pub extern "C" fn maxima_mx_create() -> *const c_void {
    let maxima_arc = Arc::new(Mutex::new(Maxima::new()));
    Arc::into_raw(maxima_arc) as *const c_void
}

/// Sets the stored token retrieved from [maxima_login].
#[no_mangle]
pub extern "C" fn maxima_mx_set_access_token(
    runtime: *mut *mut Runtime,
    mx: *mut *const c_void,
    token: *const c_char,
) -> usize {
    if mx.is_null() || token.is_null() {
        return ERR_INVALID_ARGUMENT;
    }

    unsafe {
        let maxima_arc = Arc::from_raw(*mx as *const Mutex<Maxima>);

        let rt = Box::from_raw(*runtime);
        rt.block_on(async {
            let str_buf = parse_raw_string(token);
            maxima_arc.lock().await.access_token = str_buf;
        });

        *runtime = Box::into_raw(rt);
        *mx = Arc::into_raw(maxima_arc) as *const c_void;
    }

    ERR_SUCCESS
}

/// Starts the LSX server used for game communication.
#[no_mangle]
pub extern "C" fn maxima_mx_start_lsx(runtime: *mut *mut Runtime, mx: *mut *const c_void) -> usize {
    if runtime.is_null() || mx.is_null() {
        return ERR_INVALID_ARGUMENT;
    }

    let result = unsafe {
        let maxima_arc = Arc::from_raw(*mx as *const Mutex<Maxima>);

        let rt = Box::from_raw(*runtime);
        let result =
            rt.block_on(async { maxima_arc.lock().await.start_lsx(maxima_arc.clone()).await });

        *runtime = Box::into_raw(rt);
        *mx = Arc::into_raw(maxima_arc) as *const c_void;
        result
    };

    if result.is_err() {
        return ERR_UNKNOWN;
    }

    ERR_SUCCESS
}

/// Consume pending LSX events.
#[no_mangle]
pub extern "C" fn maxima_mx_consume_lsx_events(
    runtime: *mut *mut Runtime,
    mx: *mut *const c_void,
    events_out: *mut *mut *const c_char,
    event_count_out: *mut c_uint,
) -> usize {
    if runtime.is_null() || mx.is_null() {
        return ERR_INVALID_ARGUMENT;
    }

    let events = unsafe {
        let maxima_arc = Arc::from_raw(*mx as *const Mutex<Maxima>);

        let rt = Box::from_raw(*runtime);
        let result = rt.block_on(async { maxima_arc.lock().await.consume_pending_events() });

        *runtime = Box::into_raw(rt);
        *mx = Arc::into_raw(maxima_arc) as *const c_void;
        result
    };

    let mut c_strings = Vec::with_capacity(events.len());
    for event in events.iter() {
        let lsx_request = if let MaximaEvent::ReceivedLSXRequest(r) = event {
            r
        } else {
            continue;
        };

        let name: &'static str = lsx_request.into();
        c_strings.push(CString::new(name).unwrap());
    }

    let mut raw_strings = Vec::with_capacity(c_strings.len());
    for s in c_strings {
        raw_strings.push(s.into_raw());
    }

    unsafe {
        *events_out = Box::into_raw(raw_strings.into_boxed_slice()) as *mut *const c_char;
        *event_count_out = events.len() as u32;
    }

    ERR_SUCCESS
}

/// Free LSX events retrieved from [maxima_mx_consume_lsx_events].
#[no_mangle]
pub unsafe extern "C" fn maxima_mx_free_lsx_events(events: *mut *mut c_char, event_count: c_uint) {
    let slice = slice::from_raw_parts_mut(events, event_count as usize);
    for &mut raw_str in slice.iter_mut() {
        drop(CString::from_raw(raw_str));
    }

    drop(Box::from_raw(slice));
}

/// Launch a game with Maxima, providing an EA Offer ID.
#[no_mangle]
pub extern "C" fn maxima_launch_game(
    runtime: *mut *mut Runtime,
    mx: *mut *const c_void,
    c_offer_id: *const c_char,
) -> usize {
    if runtime.is_null() || mx.is_null() || c_offer_id.is_null() {
        return ERR_INVALID_ARGUMENT;
    }

    let result = unsafe {
        let maxima_arc = Arc::from_raw(*mx as *const Mutex<Maxima>);

        let rt = Box::from_raw(*runtime);
        let result = rt.block_on(async {
            let offer_id = parse_raw_string(c_offer_id);
            launch::start_game(&offer_id, None, vec![], maxima_arc.clone()).await
        });

        *runtime = Box::into_raw(rt);
        *mx = Arc::into_raw(maxima_arc) as *const c_void;
        result
    };

    if result.is_err() {
        return ERR_UNKNOWN;
    }

    ERR_SUCCESS
}

/// Send a request to the EA Service Layer
fn maxima_send_service_request<T, R>(
    runtime: *mut *mut Runtime,
    mx: *mut *const c_void,
    token: *const c_char,
    operation: &GraphQLRequest,
    variables: T,
    response_out: *mut R,
) -> usize
where
    T: Serialize,
    R: for<'a> Deserialize<'a>,
{
    if runtime.is_null() || mx.is_null() || token.is_null() {
        return ERR_INVALID_ARGUMENT;
    }

    let result = unsafe {
        let maxima_arc = Arc::from_raw(*mx as *const Mutex<Maxima>);

        let rt = Box::from_raw(*runtime);
        let result = rt.block_on(async {
            let token = parse_raw_string(token);
            send_service_request::<T, R>(&token, operation, variables).await
        });

        *runtime = Box::into_raw(rt);
        *mx = Arc::into_raw(maxima_arc) as *const c_void;
        result
    };

    if result.is_err() {
        return ERR_UNKNOWN;
    }

    ERR_SUCCESS
}

// TODO: Need to find a good way to do this
/* #[no_mangle]
pub extern "C" fn maxima_service_layer_get_user_player(
    runtime: *mut *mut Runtime,
    mx: *mut *const c_void,
    token: *const c_char,
    response_out: *mut maxima::core::service_layer::$response,
) -> usize {
    use maxima::core::service_layer::[<SERVICE_REQUEST_ $operation:upper>];
    maxima_send_service_request(runtime, mx, token, [<SERVICE_REQUEST_ $operation:upper>], variables, response_out)
}

define_native_service_request!(GetBasicPlayer, ServiceGetBasicPlayerRequest, ServicePlayer);

macro_rules! define_native_service_request {
    ($operation:expr, $request:ident, $response:ident) => {
        paste::paste! {
            #[no_mangle]
            pub extern "C" fn [<maxima_send_service_request_ $operation:snake:lower>](
                runtime: *mut *mut Runtime,
                mx: *mut *const c_void,
                token: *const c_char,
                variables: maxima::core::service_layer::$request,
                response_out: *mut maxima::core::service_layer::$response,
            ) -> usize {
                use maxima::core::service_layer::[<SERVICE_REQUEST_ $operation:upper>];
                maxima_send_service_request(runtime, mx, token, [<SERVICE_REQUEST_ $operation:upper>], variables, response_out)
            }
        }
    };
} */

unsafe fn parse_raw_string(buf: *const c_char) -> String {
    let c_str = CStr::from_ptr(buf);
    let str_slice = c_str.to_str().unwrap();
    str_slice.to_owned()
}
