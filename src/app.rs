use leptos::{ev::SubmitEvent, html::Input, *};
use leptos_meta::*;
use leptos_router::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

const LINE_ITEM_STORAGE_KEY: &str = "line-item-storage";
const PARTICIPANTS_STORAGE_KEY: &str  = "participants-storage";
const EVENT_STORAGE_KEY: &str  = "event-item-storage";


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Participants(pub Vec<Participant>);

impl Participants {
    pub fn new() -> Self {
        let starting_participants =
            window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|storage| {
                    storage.get_item(PARTICIPANTS_STORAGE_KEY).ok().flatten().and_then(
                        |value| serde_json::from_str::<Vec<Participant>>(&value).ok(),
                    )
                })
            .unwrap_or_default();
        Self(starting_participants)
    }


    pub fn is_empty(&self) -> bool { self.0.is_empty()
    }

    pub fn add(&mut self, participant: Participant) {
        self.0.push(participant);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.retain(|participant| participant.id != id)
    }


    fn retain(&mut self, mut f: impl FnMut(&Participant) -> bool) {
        self.0.retain(|participant| {
            let retain = f(participant);
            if !retain {

                participant.name.dispose();
            }
            retain
        })

    }
}

impl Default for Participants {
    fn default() -> Self {
        Self::new()
    }
}



#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: RwSignal<String>,
}


impl Participant {
    pub fn new(id: Uuid, name: String) -> Self {
        let name = create_rw_signal(name);
        Self {
            id,
            name
        }
    }
}


#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (participants, set_participants) = create_signal(Participants::new());
    provide_context(set_participants);
    provide_context(participants);


    create_effect(move|_| {
        if let Ok(Some(storage)) = window().local_storage() {
            let json = serde_json::to_string(&participants)
                .expect("Couldn't serialize participants");
            if storage.set_item(PARTICIPANTS_STORAGE_KEY, &json).is_err() {
                log::error!("Error while trying to set item in local storage")
            }
        }
    });

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

    let participants = use_context::<ReadSignal<Participants>>().unwrap();

    view! {
        <body class="bg-gray-100">
            <main class="container mx-auto p-4">
                <section class="bg-white p-8 rounded-lg shadow-md max-w-md mx-auto mt-8">
                    <div class="mb-4">
                        <div>Whati8 phase-1</div>
                        <input type="text" id="e-name" placeholder="Enter the event name.." class="mt-2 p-2 border rounded-md w-full" />
                    </div>
                    <label for="e-date">Enter event date</label>
                    <input type="date" id="e-date" class="mt-2 p-2 border rounded-md w-full" />
                    <label for="e-tax">Enter tax amount</label>
                    <input type="double" id="e-tax" placeholder="Enter the tax amount" class="mt-2 p-2 border rounded-md w-full" />
                    <label for="e-amount">Enter total amount</label>
                    <input type="double" id="e-amount" placeholder="Enter the total amount" class="mt-2 p-2 border rounded-md w-full" />
                    <ParticipantAdder/>

                    {participants.get().0.to_vec().clone().iter()
                            .map(|n| view!{
                                <div >{n.name.get().to_string()}</div>
                            })
                            .collect_view()}

                </section>
            </main>
        </body>
    }
}



#[component]
fn ParticipantAdder() -> impl IntoView {
    let set_participants = use_context::<WriteSignal<Participants>>().unwrap();
    let participant_input_ref = create_node_ref::<Input>();
    let add_participant = move |_| {
        let input = participant_input_ref.get().unwrap();
        let name = input.value();
        let name = name.trim();
        if !name.is_empty() {
            let new = Participant::new(
                Uuid::new_v4(),
                name.to_string()
            );
            set_participants.update(|p| p.add(new));
            input.set_value("");
        }
    };

    view! {
    <input name="participants" type="text" node_ref=participant_input_ref name="participants"></input>
    <button id="addButton" on:click=add_participant>Add</button>
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
