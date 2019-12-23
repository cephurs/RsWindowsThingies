use std::ffi::OsString;
use std::ptr::null_mut;
use winapi::um::winevt::*;
use winapi::ctypes::c_void;
use std::os::windows::prelude::*;
use winapi::shared::minwindef::{DWORD};
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER;


/// BOOL EvtRender(
///   EVT_HANDLE Context,
///   EVT_HANDLE Fragment,
///   DWORD      Flags,
///   DWORD      BufferSize,
///   PVOID      Buffer,
///   PDWORD     BufferUsed,
///   PDWORD     PropertyCount
/// );
pub fn evt_render(event_handle: EVT_HANDLE) -> Option<String> {
    let mut buffer_used: DWORD = 0;
    let mut property_count: DWORD = 0;

    let context = null_mut();
    let flags = EvtRenderEventXml;

    let result = unsafe {
        EvtRender(
            context,
            event_handle as _,
            flags,
            0,
            null_mut(),
            &mut buffer_used,
            &mut property_count
        )
    };

    // We expect this to fail but return the buffer size needed.
    if result == 0 {
        let last_error: DWORD = unsafe {
            GetLastError()
        };

        if last_error == ERROR_INSUFFICIENT_BUFFER {
            let buffer: Vec<u16> = vec![0; buffer_used as usize];

            let result = unsafe {
                EvtRender(
                    context,
                    event_handle as _,
                    flags,
                    buffer.len() as _,
                    buffer.as_ptr() as _,
                    &mut buffer_used,
                    &mut property_count
                )
            };

            if result != 0 {
                let mut index = buffer_used as usize - 1;

                // Buffers can be null padded. We want to trim the null chars.
                match buffer.iter().position(|&x| x == 0) {
                    Some(i) => {
                        index = i;
                    },
                    None => {}
                }

                let xml_string = OsString::from_wide(
                    &buffer[..index]
                ).to_string_lossy().to_string();

                return Some(xml_string);
            }
        }
    }

    None
}


/// DWORD EvtSubscribeCallback(
///   EVT_SUBSCRIBE_NOTIFY_ACTION Action,
///   PVOID UserContext,
///   EVT_HANDLE Event
/// )
pub extern "system" fn evt_subscribe_callback(
    action: EVT_SUBSCRIBE_NOTIFY_ACTION, 
    _user_context: *mut c_void, 
    event_handle: EVT_HANDLE
) -> u32 {
    if action != EvtSubscribeActionDeliver {
        error!("Expected EvtSubscribeActionDeliver for evt_subscribe_callback but found {:?}", action);
        return 0;
    }

    match evt_render(event_handle) {
        Some(xml_event) => {
            println!("{}", xml_event);
        },
        None => {}
    }

    return 0;
}


/// EVT_HANDLE EvtSubscribe(
///   EVT_HANDLE             Session,
///   HANDLE                 SignalEvent,
///   LPCWSTR                ChannelPath,
///   LPCWSTR                Query,
///   EVT_HANDLE             Bookmark,
///   PVOID                  Context,
///   EVT_SUBSCRIBE_CALLBACK Callback,
///   DWORD                  Flags
/// );
pub fn register_event_callback(
        channel_path: &String, 
        query: Option<String>
) {
    // Currently we are not implementing sessions
    let session = null_mut();
    // This is null becuase we are using a callback
    let signal_event = null_mut();

    // Create the wide string buffer
    let mut channel_path_u16 : Vec<u16> = channel_path.encode_utf16().collect();
    channel_path_u16.resize(channel_path.len() + 1, 0);

    // Get the query string, or if None was passed, make it *
    let query_str = match query {
        Some(q) => q,
        None => "*".to_owned()
    };
    let mut query_str_u16 : Vec<u16> = query_str.encode_utf16().collect();
    query_str_u16.resize(query_str.len() + 1, 0);

    // Bookmarks are not currently implemented
    let bookmark = null_mut();

    let context = null_mut();

    let flags = EvtSubscribeToFutureEvents;

    // This handle will need to be closed when the subscription is done...
    let _subscription_handle = unsafe {
        EvtSubscribe(
            session,
            signal_event,
            channel_path_u16.as_ptr(),
            query_str_u16.as_ptr(),
            bookmark,
            context,
            Some(evt_subscribe_callback),
            flags
        )
    };
}