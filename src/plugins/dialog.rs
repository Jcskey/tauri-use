use std::path::PathBuf;
use reactive_graph::{
    effect::Effect, 
    owner::LocalStorage, 
    signal::{signal, WriteSignal}, 
    traits::{Get as _, GetUntracked as _,  Set as _, UpdateUntracked as _}, 
    wrappers::read::Signal
};
use serde::{Deserialize, Serialize};

use crate::{use_invoke_with_args, use_invoke_with_options, UseTauriError, UseTauriWithReturn};

pub fn use_ask_dialog<T>() -> UseDialogReturn<ConfirmDialogOpions, T>
where 
    T: Clone + Send + Sync + 'static
{
    let (transfer, set_transfer) = signal(None::<T>);
    let (dialog_options, set_dialog_options) = signal(None::<(ConfirmDialogOpions, T)>);

    let UseTauriWithReturn {
        data,
        error,
        trigger,
    } = use_invoke_with_args::<ConfirmDialogOpions, bool>("plugin:dialog|ask");

    Effect::new(move || {
        if let Some((options, _)) = dialog_options.get() {
            trigger.set(Some(options));
        }
    });
    
    Effect::new(move || {
        if let Some(b) = data.get() {
            if b {
                set_transfer.set(Some(dialog_options.get_untracked().unwrap().1));
            }
            set_dialog_options.update_untracked(|v| *v = None);
        }
    });

    Effect::new(move || {
        if let Some(_) = error.get() {
            set_dialog_options.update_untracked(|v| *v = None);
        }
    });

    UseDialogReturn { 
        transfer: transfer.into(), 
        error: error.into(), 
        set_dialog_options: set_dialog_options,
    }
}

pub fn use_confirm_dialog<T>() -> UseDialogReturn<ConfirmDialogOpions, T>
where 
    T: Clone + Send + Sync + 'static
{
    let (transfer, set_transfer) = signal(None::<T>);
    let (dialog_options, set_dialog_options) = signal(None::<(ConfirmDialogOpions, T)>);

    let UseTauriWithReturn {
        data,
        error,
        trigger,
    } = use_invoke_with_args::<ConfirmDialogOpions, bool>("plugin:dialog|confirm");

    Effect::new(move || {
        if let Some((options, _)) = dialog_options.get() {
            trigger.set(Some(options));
        }

        if let Some(b) = data.get() {
            if b {
                set_transfer.set(Some(dialog_options.get_untracked().unwrap().1));
            }
            set_dialog_options.update_untracked(|v| *v = None);
        }

        if let Some(_) = error.get() {
            set_dialog_options.update_untracked(|v| *v = None);
        }
    });

    UseDialogReturn { 
        transfer: transfer.into(), 
        error: error.into(), 
        set_dialog_options: set_dialog_options,
    }
}

pub fn use_message_dialog<T>() -> UseDialogReturn<MessageDialogOpions, T>
where 
    T: Clone + Send + Sync + 'static
{
    let (transfer, set_transfer) = signal(None::<T>);
    let (dialog_options, set_dialog_options) = signal(None::<(MessageDialogOpions, T)>);

    let UseTauriWithReturn {
        data,
        error,
        trigger,
    } = use_invoke_with_args::<MessageDialogOpions, ()>("plugin:dialog|message");

    Effect::new(move || {
        if let Some((options, _)) = dialog_options.get() {
            trigger.set(Some(options));
        }

        if let Some(()) = data.get() {
            set_transfer.set(Some(dialog_options.get_untracked().unwrap().1));
            set_dialog_options.update_untracked(|v| *v = None);
        }

        if let Some(_) = error.get() {
            set_dialog_options.update_untracked(|v| *v = None);
        }
    });

    UseDialogReturn { 
        transfer: transfer.into(), 
        error: error.into(), 
        set_dialog_options: set_dialog_options,
    }
}

pub fn use_open_dialog() -> UseTauriWithReturn<OpenDialogOptions, Option<OpenDialogReturn>> {
    #[derive(Clone, Serialize)]
    struct OptionsWrapper {
        options: OpenDialogOptions,
    }

    let (args_wrapper, set_args_wrapper) = signal(None::<OpenDialogOptions>);

    let UseTauriWithReturn { 
        data, 
        error, 
        trigger 
    } = use_invoke_with_args::<OptionsWrapper, Option<OpenDialogReturn>>("plugin:dialog|open");

    Effect::new(move || {
        if let Some(options) = args_wrapper.get() {
            trigger.set(Some(OptionsWrapper { options }));
            set_args_wrapper.update_untracked(|v| *v = None);
        }
    });

    UseTauriWithReturn { 
        data: data.into(), 
        error: error.into(), 
        trigger: set_args_wrapper,
    }
}

pub fn use_save_dialog() -> UseTauriWithReturn<SaveDialogOptions, Option<PathBuf>> {
    use_invoke_with_options::<SaveDialogOptions, Option<PathBuf>>("plugin:dialog|save")
}

pub struct UseDialogReturn<O, T> 
where 
    O: serde::Serialize + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub transfer: Signal<Option<T>>,
    pub error: Signal<Option<UseTauriError>, LocalStorage>,
    pub set_dialog_options: WriteSignal<Option<(O, T)>>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum OpenDialogReturn {
    Files(Vec<String>),
    File(String)
}

#[derive(Clone, Default, Serialize)]
#[serde(rename = "camelCase")]
pub struct ConfirmDialogOpions {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<MessageDialogKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok_label: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_label: Option<&'static str>
}

impl ConfirmDialogOpions {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Serialize)]
#[serde(rename = "camelCase")]
pub struct MessageDialogOpions {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<MessageDialogKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok_label: Option<&'static str>,
}

impl MessageDialogOpions {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Serialize)]
#[serde(untagged, rename = "camelCase")]
pub enum MessageDialogKind {
    #[default]
    Info,
    Warning,
    Error
}

#[derive(Clone, Default, Serialize)]
#[serde(rename = "camelCase")]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename = "camelCase")]
pub struct OpenDialogOptions {
    pub title: Option<String>,
    pub filters: Vec<DialogFilter>,
    pub multiple: bool,
    pub directory: bool,
    pub default_path: Option<PathBuf>,
    pub recursive: bool,
    pub can_create_directories: bool
}

#[derive(Clone, Default, Serialize)]
#[serde(rename = "camelCase")]
pub struct SaveDialogOptions {
    pub title: Option<String>,
    pub filters: Vec<DialogFilter>,
    pub default_path: Option<PathBuf>,
    pub can_create_directories: bool
}
