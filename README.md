# Tauri use

Inspired by `leptos-use`, designed for working with Tauri's native APIs..

<br/>

## Usage
1. Invoke Function

```rust
use leptos::prelude::*;
use tauri_use::{use_invoke_with_args, UseTauriWithReturn};

#[component]
pub fn Demo() -> impl IntoView {
    let UseTauriWithReturn { 
        data: created_demo_data,
        trigger: create_demo ,
        ..
    } = use_invoke_with_args::<DemoModelWrapper, Demo>("create_demo");

    let on_create = move |_| {
        // some invalid code
        // ...
        // ...
        create_demo.set(Some(DemoModelWrapper::new(...)));
    }

    view! {
        ...
        <button on:click=on_create>create</botton>
        ...
    }
}
```

2. Listen Function

```rust
use leptos::prelude::*;
use tauri_use::{use_listen, UseListenReturn, EventType};

#[component]
pub fn Demo() -> impl IntoView {
    let UseListenReturn {
        data: demo_status,
        open,
        close,
        ..
    } = use_listen::<Vec<DemoStatus>>(EventType::Custom("app://demo_status"));

    let on_open = move |_| {
        open()
    };

    let on_close = move |_| {
        close()
    }

    view! {
        ...
        <button on:click=on_close>close</button>
        <button on:click=on_open>open</button>
        ...
    }
}
```
