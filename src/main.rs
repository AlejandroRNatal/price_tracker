use leptos::prelude::*;
use leptos::mount::mount_to_body;


#[component]
fn Table() -> impl IntoView {
    let (rows, set_rows) =  signal::<Vec<(usize, &str, &str, &str)>>(vec![
        (1, "Pikachy ex", "0.10","2025-06-07")
    ]);

    view!{
        <table>
            <tr>
                <th>Name</th>
                <th>Price</th>
                <th>Date</th>
            </tr>
            <For
                each= move || rows.get()
                key=|row| row.0
                children=move |row: (usize, &str, &str, &str)| {
                    view! {
                        <tr>
                            <th>{row.1}</th>
                            <th>{row.2}</th>
                            <th>{row.3}</th>
                        </tr>
                    }
                }
            />
        </table>
    }
}

pub fn main() {
    //_ = console_log::init_with_level(log::Level::Debug);
    //console_error_panic_hook::set_once();
    mount_to_body(Table);
}
