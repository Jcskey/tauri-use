# Tauri use

<br/>

## Usage
```rust
use leptos::prelude::*;
use tauri_use::{use_invoke_with_args, UseTauriWithReturn};

#[component]
pub fn Demo() -> impl IntoView {
    let UseTauriWithReturn { 
        data: create_demo_data,
        trigger: create_demo ,
        ..
    } = use_invoke_with_args::<DemoModelWrapper, Demo>("create_demo");

    view! {
        ..
    }
}
```
