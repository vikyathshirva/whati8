use leptos::{ev::SubmitEvent, html::Input, *};
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <link href="https://cdnjs.cloudflare.com/ajax/libs/flowbite/2.2.1/flowbite.min.css"  rel="stylesheet" />

        // sets the document title
        <Title text="Whati8"/>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/flowbite/2.2.1/flowbite.min.js"></script>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=Homepage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}


#[component]
fn Homepage() -> impl IntoView {
    let (eventName, setEventName) = create_signal(String::new());
    let (eventDate, setEventDate) = create_signal(String::new());
    let (participants, setParticipants) = create_signal(Vec<String>::new());
    let (items, setItems) = create_signal(Vec::<(String, String, Vec<String>)>::new());
    let (additionalTax, setAdditionalTax) = create_signal(String::new());
    let (selectedParticipants, setSelectedParticipants) = create_signal(Vec::new());

    view! {
    }


}


/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
