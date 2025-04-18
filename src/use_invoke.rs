use core::fmt;

use reactive_graph::{
    effect::Effect, 
    owner::{LocalStorage, StoredValue}, 
    signal::{signal, signal_local, WriteSignal}, 
    spawn_local_scoped, 
    traits::{Get as _, GetValue as _, Set as _, UpdateUntracked as _}, 
    wrappers::read::Signal
};
use wasm_bindgen::prelude::*;

pub fn use_command<T>(
    cmd: &'static str,
) -> UseTauriWithReturn<(), T> 
where 
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let (args, set_args) = signal(None::<()>);

    let UseTauriReturn { 
        data, 
        error, 
        trigger 
    } = use_invoke::<(), (), T>(cmd);

    Effect::new(move || {
        if let Some(()) = args.get() {
            trigger.set(Some(((), ())));
            set_args.update_untracked(|v| *v = None);
        }
    });

    UseTauriWithReturn { 
        data: data.into(), 
        error: error.into(), 
        trigger: set_args
    }
}

pub fn use_invoke_with_args<Args, T>(
    cmd: &'static str,
) -> UseTauriWithReturn<Args, T> 
where 
    Args: serde::Serialize + Clone + Send + Sync + 'static,
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let (args, set_args) = signal(None::<Args>);

    let UseTauriReturn { 
        data, 
        error, 
        trigger 
    } = use_invoke::<Args, (), T>(cmd);

    Effect::new(move || {
        if let Some(args) = args.get() {
            trigger.set(Some((args, ())));
            set_args.update_untracked(|v| *v = None);
        }
    });

    UseTauriWithReturn { 
        data: data.into(), 
        error: error.into(), 
        trigger: set_args
    }
}

pub fn use_invoke_with_options<Opts, T>(
    cmd: &'static str,
) -> UseTauriWithReturn<Opts, T> 
where 
    Opts: serde::Serialize + Clone + Send + Sync + 'static,
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let (opts, set_opts) = signal(None::<Opts>);

    let UseTauriReturn { 
        data, 
        error, 
        trigger 
    } = use_invoke::<(), Opts, T>(cmd);

    Effect::new(move || {
        if let Some(opts) = opts.get() {
            trigger.set(Some(((), opts)));
            set_opts.update_untracked(|v| *v = None);
        }
    });

    UseTauriWithReturn { 
        data: data.into(), 
        error: error.into(), 
        trigger: set_opts
    }
}

pub fn use_invoke<Args, Opts, T>(
    cmd: &'static str
) -> UseTauriReturn<Args, Opts, T> 
where 
    Args: serde::Serialize + Clone + Send + Sync + 'static,
    Opts: serde::Serialize + Clone + Send + Sync + 'static,
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let (data, set_data) = signal(None::<T>);
    let (error, set_error) = signal_local(None::<UseTauriError>);
    let (trigger, set_trigger) = signal(None::<(Args, Opts)>);

    let init = StoredValue::new(move |cmd: &'static str, args: Args, options: Opts| {
        #[derive(Clone, serde::Serialize)]
        struct OptionsWrapper<Opts>
        where 
            Opts: serde::Serialize + Clone + 'static,
        {
            options: Opts
        }

        let args = args.clone();
        let options = OptionsWrapper{
            options: options.clone()
        };

        spawn_local_scoped(async move {
            let args = match serde_wasm_bindgen::to_value(&args) {
                Ok(value) => value,
                Err(err) =>  {
                    set_error.set(Some(UseTauriError::Serialize(err.to_string())));
                    return;
                }
            };

            let options = match serde_wasm_bindgen::to_value(&options) {
                Ok(value) => value,
                Err(err) =>  {
                    set_error.set(Some(UseTauriError::Serialize(err.to_string())));
                    return;
                }
            };

            match invoke(cmd, args, options).await {
                Ok(data) => {
                    match serde_wasm_bindgen::from_value::<T>(data) {
                        Ok(data) => set_data.set(Some(data)),
                        Err(err) => set_error.set(Some(UseTauriError::Deserialize(err.to_string())))
                    }
                }
                Err(err) => {
                    let err_str = err.as_string().unwrap_or_else(|| "Unknown error".to_string());
                    set_error.set(Some(UseTauriError::Command(cmd, err_str)))
                }
            }
        });
    });
    
    Effect::new(move || {
        let init = init.get_value();
        if let Some((args, opts)) = trigger.get() {
            init(cmd, args, opts);
        }
        set_trigger.update_untracked(|v| *v = None);
    });

    UseTauriReturn { 
        data: data.into(), 
        error: error.into(),
        trigger: set_trigger
    }
}



pub struct UseTauriReturn<Args, Opts, T>
where 
    Args: serde::Serialize + Clone + 'static,
    Opts: serde::Serialize + Clone + 'static,
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<UseTauriError>, LocalStorage>,
    pub trigger: WriteSignal<Option<(Args, Opts)>>,
}

pub struct UseTauriWithReturn<O, T>
where 
    O: serde::Serialize + Clone + 'static,
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<UseTauriError>, LocalStorage>,
    pub trigger: WriteSignal<Option<O>>,
}

#[derive(Clone, Debug)]
pub enum UseTauriError {
    Command(&'static str, String),
    Serialize(String),
    Deserialize(String),
}

impl fmt::Display for UseTauriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
            UseTauriError::Command(place, err) => write!(f, "Command error in {}: {}", place, err),
            UseTauriError::Serialize(err) => write!(f, "Error serializing value: {}", err),
            UseTauriError::Deserialize(err) => write!(f, "Error deserializing value: {}", err),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue, options: JsValue) -> Result<JsValue, JsValue>;
}
