use core::fmt;
use std::sync::{atomic::AtomicBool, Arc};
use reactive_graph::{
    owner::{on_cleanup, LocalStorage, StoredValue}, 
    signal::{signal, signal_local}, 
    spawn_local_scoped, 
    traits::{GetUntracked as _, GetValue as _, Set as _, SetValue as _}, 
    wrappers::read::Signal
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use js_sys::Function;
use wasm_bindgen::prelude::*;

pub fn use_listen<T>(
    event: EventType,
) -> UseListenReturn<
    T,
    impl Fn() + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
>
where 
    T: DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
{
    use_listen_inner::<T, _>(event.as_str(), listen, EventTarget::default())
}

pub fn use_listen_with_target<T>(
    event: EventType,
    target: EventTarget
) -> UseListenReturn<
    T,
    impl Fn() + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
>
where 
    T: DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
{
    use_listen_inner::<T, _>(event.as_str(), once, target)
}

pub fn use_once_listen<T>(
    event: EventType,
) -> UseListenReturn<
    T,
    impl Fn() + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
>
where 
    T: DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
{
    use_listen_inner::<T, _>(event.as_str(), once, EventTarget::default())
}

pub fn use_once_listen_with_target<T>(
    event: EventType,
    target: EventTarget
) -> UseListenReturn<
    T,
    impl Fn() + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
>
where 
    T: DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
{
    use_listen_inner::<T, _>(event.as_str(), once, target)
}

fn use_listen_inner<T, F>(
    event: &'static str,
    callback: F,
    target: EventTarget,
) -> UseListenReturn<
    T,
    impl Fn() + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
>
where
    T: DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
    F: AsyncFn(&str, &Closure<dyn Fn(JsValue)>, JsValue) -> Result<JsValue, JsValue> + Send + Sync + 'static,
{
    let (event_id, set_event_id) = signal(None::<u32>);
    let (data, set_data) = signal(None::<T>);
    let (error, set_error) = signal_local(None::<UseListenError>);
    let (unlisten, set_unlisten) = signal_local(None::<Arc<dyn Fn() + Send + Sync>>);
    let explicitly_closed = Arc::new(AtomicBool::new(false));
    let callback = Arc::new(callback);

    let close = {
        let explicitly_closed = Arc::clone(&explicitly_closed);

        let wrapped = send_wrapper::SendWrapper::new(move || {
            if let Some(unlisten) = unlisten.get_untracked() {
                unlisten();
                set_unlisten.set(None);
                explicitly_closed.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });

        move || {
            wrapped()
        }
    };

    let init = StoredValue::new(None::<Arc<dyn Fn() + Send + Sync>>);
    

    init.set_value(Some(Arc::new({
        let explicitly_closed = Arc::clone(&explicitly_closed);

        move || {
            use wasm_bindgen::prelude::*;

            if explicitly_closed.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let on_msg = Closure::wrap(Box::new(move |event: JsValue| {
                match serde_wasm_bindgen::from_value::<Event<T>>(event) {
                    Ok(evt) => {
                        set_data.set(Some(evt.payload));
                        set_event_id.set(Some(evt.id));
                    }
                    Err(err) => {
                        set_error.set(Some(UseListenError::Deserialize(err.to_string())))
                    }
                }
            }) as Box<dyn Fn(JsValue)>);

            let callback = callback.clone();
            
            spawn_local_scoped(async move {
                let options = match serde_wasm_bindgen::to_value(&target) {
                    Ok(opt) => opt,
                    Err(err) => {
                        set_error.set(Some(UseListenError::Serialize(err.to_string())));
                        return;
                    }
                };

                match callback(event, &on_msg, options).await {
                    Ok(unlisten) => {
                        let wrapped = send_wrapper::SendWrapper::new(move || {
                            let call: &Function = unlisten.unchecked_ref();
                            js_sys::Function::apply(call, &JsValue::NULL, &js_sys::Array::new()).unwrap();
                        });
                        set_unlisten.set(Some(Arc::new(move || {
                            wrapped()
                        })));
                    }
                    Err(err) => {
                        let err_str = err.as_string().unwrap_or_else(|| "Unknown error".to_string());
                        set_error.set(Some(UseListenError::Event(event, err_str)))
                    }
                }
                on_msg.forget();
            });
        }
    })));

    let open = {
        let close = close.clone();
        let explicity_closed = Arc::clone(&explicitly_closed);

        let wrapped = send_wrapper::SendWrapper::new(move || {
            close();
            explicity_closed.store(false, std::sync::atomic::Ordering::Relaxed);
            if let Some(init) = init.get_value() {
                init();
            }
        });
        move || {
            wrapped()
        }
    };

    on_cleanup(close.clone());

    UseListenReturn { 
        data: data.into(), 
        event_id: event_id.into(),
        error: error.into(), 
        open, 
        close,
    }
} 

// reference: https://github.com/tauri-apps/tauri/blob/dev/packages/api/src/event.ts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    WindowResized,
    WindowMoved,
    WindowCloseRequested,
    WindowDestroyed,
    WindowFocus,
    WindowBlur,
    WindowScaleFactorChanged,
    WindowThemeChanged,
    WindowCreated,
    WebViewCreated,
    DragEnter,
    DragOver,
    DragDrop,
    DragLeave,
    Custom(&'static str)
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::WindowResized => "tauri://resize",
            EventType::WindowMoved => "tauri://move",
            EventType::WindowCloseRequested => "tauri://close-requested",
            EventType::WindowDestroyed => "tauri://destroyed",
            EventType::WindowFocus => "tauri://focus",
            EventType::WindowBlur => "tauri://blur",
            EventType::WindowScaleFactorChanged => "tauri://scale-change",
            EventType::WindowThemeChanged => "tauri://theme-changed",
            EventType::WindowCreated => "tauri://window-created",
            EventType::WebViewCreated => "tauri://webview-created",
            EventType::DragEnter => "tauri://drag-enter",
            EventType::DragOver => "tauri://drag-over",
            EventType::DragDrop => "tauri://drag-drop",
            EventType::DragLeave => "tauri://drag-leave",
            EventType::Custom(s) => {
                if  s.chars()
                    .all(|c| c.is_alphabetic() || c == '-' || c == '/' || c == ':' || c == '_') {
                        s
                    } else {
                        panic!("Event name must include only alphanumeric characters, `-`, `/`, `:` and `_`.")
                    }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "label")]
#[non_exhaustive]
pub enum EventTarget {
    #[default]
    Any,
    AnyLabel(&'static str),
    App,
    Window(&'static str),
    Webview(&'static str),
    WebviewWindow(&'static str)
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Event<T> {
    pub event: String,
    pub id: u32,
    pub payload: T,
}

pub struct UseListenReturn<T, OpenFn, CloseFn>
where 
    T: DeserializeOwned + Clone + Send + Sync + 'static,
    OpenFn: Fn() + Clone + Send + Sync + 'static,
    CloseFn: Fn() + Clone + Send + Sync + 'static,
{
    pub data: Signal<Option<T>>,
    pub event_id: Signal<Option<u32>>,
    pub error: Signal<Option<UseListenError>, LocalStorage>,
    pub open: OpenFn,
    pub close: CloseFn,
}

#[derive(Clone, Debug)]
pub enum UseListenError {
    Event(&'static str, String),
    Serialize(String),
    Deserialize(String),
}

impl fmt::Display for UseListenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
            UseListenError::Event(place, err) => write!(f, "Event error in {}: {}", place, err),
            UseListenError::Serialize(err) => write!(f, "Error serializing value: {}", err),
            UseListenError::Deserialize(err) => write!(f, "Error deserializing value: {}", err),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        closure: &Closure<dyn Fn(JsValue)>,
        option: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    async fn once(
        event: &str,
        closure: &Closure<dyn Fn(JsValue)>,
        option: JsValue,
    ) -> Result<JsValue, JsValue>;
}
